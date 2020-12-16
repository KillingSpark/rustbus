//! The connection stuff you probably want to use. Conn is the lowlevel abstraction RpcConn is the higher level wrapper with convenience functions
//! over the Conn struct.

use super::ll_conn::DuplexConn;
use super::*;
use crate::message_builder::MarshalledMessage;
use crate::message_builder::MessageType;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time;

/// Convenience wrapper around the lowlevel connection
pub struct RpcConn {
    signals: VecDeque<MarshalledMessage>,
    calls: VecDeque<MarshalledMessage>,
    responses: HashMap<u32, MarshalledMessage>,
    conn: DuplexConn,
    filter: MessageFilter,
}

/// Filter out messages you dont want in your RpcConn.
/// If this filters out a call, the RpcConn will send a UnknownMethod error to the caller. Other messages are just dropped
/// if the filter returns false.
/// ```rust,no_run
/// use rustbus::{get_session_bus_path, standard_messages, DuplexConn, params::Container, params::DictMap, MessageBuilder, MessageType, RpcConn};
///
/// fn main() -> Result<(), rustbus::connection::Error> {
///     let session_path = get_session_bus_path()?;
///     let con = DuplexConn::connect_to_bus(session_path, true)?;
///     let mut rpc_con = RpcConn::new(con);
///
///     rpc_con.set_filter(Box::new(|msg| match msg.typ {
///     MessageType::Call => {
///         let right_interface_object = msg.dynheader.object.eq(&Some("/io/killing/spark".into()))
///             && msg.dynheader.interface.eq(&Some("io.killing.spark".into()));
///
///         let right_member = if let Some(member) = &msg.dynheader.member {
///             member.eq("Echo") || member.eq("Reverse")
///         } else {
///             false
///         };
///         let keep = right_interface_object && right_member;
///         if !keep {
///             println!("Discard: {:?}", msg);
///         }
///         keep
///     }
///     MessageType::Invalid => false,
///     MessageType::Error => true,
///     MessageType::Reply => true,
///     MessageType::Signal => false,
/// }));
///
/// Ok(())
/// }
/// ```
pub type MessageFilter = Box<dyn Fn(&MarshalledMessage) -> bool + Sync + Send>;

impl RpcConn {
    pub fn new(conn: DuplexConn) -> Self {
        RpcConn {
            signals: VecDeque::new(),
            calls: VecDeque::new(),
            responses: HashMap::new(),
            conn,
            filter: Box::new(|_| true),
        }
    }
    pub fn conn(&self) -> &DuplexConn {
        &self.conn
    }
    pub fn conn_mut(&mut self) -> &mut DuplexConn {
        &mut self.conn
    }

    /// get the next new serial
    pub fn alloc_serial(&mut self) -> u32 {
        self.conn.send.alloc_serial()
    }

    pub fn session_conn(timeout: Timeout) -> Result<Self> {
        let session_path = get_session_bus_path()?;
        Self::connect_to_path(session_path, timeout)
    }

    pub fn system_conn(timeout: Timeout) -> Result<Self> {
        let session_path = get_system_bus_path()?;
        Self::connect_to_path(session_path, timeout)
    }

    pub fn connect_to_path(path: UnixAddr, timeout: Timeout) -> Result<Self> {
        let con = DuplexConn::connect_to_bus(path, true)?;
        let mut con = Self::new(con);

        let mut hello = crate::standard_messages::hello();
        let serial = con
            .send_message(&mut hello)?
            .write(timeout)
            .map_err(super::ll_conn::force_finish_on_error)?;

        con.wait_response(serial, timeout)?;
        Ok(con)
    }

    pub fn set_filter(&mut self, filter: MessageFilter) {
        self.filter = filter;
    }

    /// Return a response if one is there but dont block
    pub fn try_get_response(&mut self, serial: u32) -> Option<MarshalledMessage> {
        self.responses.remove(&serial)
    }

    /// Return a response if one is there or block until it arrives
    pub fn wait_response(&mut self, serial: u32, timeout: Timeout) -> Result<MarshalledMessage> {
        loop {
            if let Some(msg) = self.try_get_response(serial) {
                return Ok(msg);
            }
            self.refill_once(timeout)?;
        }
    }

    /// Return a signal if one is there but dont block
    pub fn try_get_signal(&mut self) -> Option<MarshalledMessage> {
        self.signals.pop_front()
    }

    /// Return a sginal if one is there or block until it arrives
    pub fn wait_signal(&mut self, timeout: Timeout) -> Result<MarshalledMessage> {
        loop {
            if let Some(msg) = self.try_get_signal() {
                return Ok(msg);
            }
            self.refill_once(timeout)?;
        }
    }

    /// Return a call if one is there but dont block
    pub fn try_get_call(&mut self) -> Option<MarshalledMessage> {
        self.calls.pop_front()
    }

    /// Return a call if one is there or block until it arrives
    pub fn wait_call(&mut self, timeout: Timeout) -> Result<MarshalledMessage> {
        loop {
            if let Some(msg) = self.try_get_call() {
                return Ok(msg);
            }
            self.refill_once(timeout)?;
        }
    }

    /// Send a message to the bus
    pub fn send_message<'a>(
        &'a mut self,
        msg: &'a mut crate::message_builder::MarshalledMessage,
    ) -> Result<super::ll_conn::SendMessageContext<'a>> {
        self.conn.send.send_message(msg)
    }

    fn insert_message_or_send_error(&mut self, msg: MarshalledMessage) -> Result<()> {
        if self.filter.as_ref()(&msg) {
            match msg.typ {
                MessageType::Call => {
                    self.calls.push_back(msg);
                }
                MessageType::Invalid => return Err(Error::UnexpectedTypeReceived),
                MessageType::Error => {
                    self.responses
                        .insert(msg.dynheader.response_serial.unwrap(), msg);
                }
                MessageType::Reply => {
                    self.responses
                        .insert(msg.dynheader.response_serial.unwrap(), msg);
                }
                MessageType::Signal => {
                    self.signals.push_back(msg);
                }
            }
        } else {
            match msg.typ {
                MessageType::Call => {
                    let mut reply = crate::standard_messages::unknown_method(&msg.dynheader);
                    self.conn
                        .send
                        .send_message(&mut reply)?
                        .write_all()
                        .map_err(ll_conn::force_finish_on_error)?;
                }
                MessageType::Invalid => return Err(Error::UnexpectedTypeReceived),
                MessageType::Error => {
                    // just drop it
                }
                MessageType::Reply => {
                    // just drop it
                }
                MessageType::Signal => {
                    // just drop it
                }
            }
        }
        Ok(())
    }

    /// This processes ONE message. This might be an ignored message. The result will tell you which
    /// if any message type was received. The message will be placed into the appropriate queue in the RpcConn.
    ///
    /// If a call is received that should be filtered out an error message is sent automatically
    pub fn try_refill_once(&mut self, timeout: Timeout) -> Result<Option<MessageType>> {
        let start_time = time::Instant::now();
        let msg = self
            .conn
            .recv
            .get_next_message(calc_timeout_left(&start_time, timeout)?)?;

        let typ = msg.typ;
        self.insert_message_or_send_error(msg)?;
        Ok(Some(typ))
    }

    /// This blocks until a new message (that should not be ignored) arrives.
    /// The message gets placed into the correct list. The Result will tell you which kind of message
    /// has been received.
    ///
    /// If calls are received that should be filtered out an error message is sent automatically
    pub fn refill_once(&mut self, timeout: Timeout) -> Result<MessageType> {
        let start_time = time::Instant::now();
        loop {
            if let Some(typ) = self.try_refill_once(calc_timeout_left(&start_time, timeout)?)? {
                break Ok(typ);
            }
        }
    }

    /// This will drain all outstanding IO on the socket, this will never block. If there is a partially received message pending
    /// it will be collected by the next call to any of the io-performing functions. For the callers convenience the Error::Timedout resulting of the
    /// EAGAIN/EWOULDBLOCK errors are converted to Ok(()) before returning, since these are expected to happen to normally exit this function.
    ///
    /// This will not send automatic error messages for calls to unknown methods because it does never block,
    /// but error replies should always be sent. For this reason replies to all filtered calls are collected and returned.
    /// The original messages are dropped immediatly, so it should keep memory usage
    /// relatively low. The caller is responsible to send these error replies over the RpcConn, at a convenient time.
    pub fn refill_all(&mut self) -> Result<Vec<crate::message_builder::MarshalledMessage>> {
        let mut filtered_out = Vec::new();
        loop {
            //  break if the call would block (aka no more io is possible), or return if an actual error occured
            let msg = match self.conn.recv.get_next_message(Timeout::Nonblock) {
                Err(Error::TimedOut) => break,
                Err(e) => return Err(e),
                Ok(m) => m,
            };
            if self.filter.as_ref()(&msg) {
                match msg.typ {
                    MessageType::Call => {
                        self.calls.push_back(msg);
                    }
                    MessageType::Invalid => return Err(Error::UnexpectedTypeReceived),
                    MessageType::Error => {
                        self.responses
                            .insert(msg.dynheader.response_serial.unwrap(), msg);
                    }
                    MessageType::Reply => {
                        self.responses
                            .insert(msg.dynheader.response_serial.unwrap(), msg);
                    }
                    MessageType::Signal => {
                        self.signals.push_back(msg);
                    }
                }
            } else {
                match msg.typ {
                    MessageType::Call => {
                        let reply = crate::standard_messages::unknown_method(&msg.dynheader);
                        filtered_out.push(reply);
                        // drop message but keep reply
                    }
                    MessageType::Invalid => return Err(Error::UnexpectedTypeReceived),
                    MessageType::Error => {
                        // just drop it
                    }
                    MessageType::Reply => {
                        // just drop it
                    }
                    MessageType::Signal => {
                        // just drop it
                    }
                }
            }
        }
        Ok(filtered_out)
    }
}
