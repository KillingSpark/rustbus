use super::{Error, Result, Timeout};
use crate::auth;
use crate::message_builder::MarshalledMessage;
use crate::wire::errors::UnmarshalError;
use crate::wire::{marshal, unmarshal, UnixFd};

use std::io::{self, IoSlice, IoSliceMut};
use std::os::fd::AsFd;
use std::time;

use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixStream;

use nix::cmsg_space;
use nix::sys::socket::{
    self, connect, recvmsg, sendmsg, socket, ControlMessage, ControlMessageOwned, MsgFlags,
    SockaddrStorage, UnixAddr,
};

/// A lowlevel abstraction over the raw unix socket
#[derive(Debug)]
pub struct SendConn {
    stream: UnixStream,
    header_buf: Vec<u8>,

    serial_counter: u32,
}

pub struct RecvConn {
    stream: UnixStream,

    msg_buf_in: IncomingBuffer,
    fds_in: Vec<UnixFd>,
    cmsgspace: Vec<u8>,
}

pub struct DuplexConn {
    pub send: SendConn,
    pub recv: RecvConn,
}

struct IncomingBuffer {
    buf: Vec<u8>,
    filled: usize,
}

impl IncomingBuffer {
    fn new() -> Self {
        IncomingBuffer {
            buf: Vec::new(),
            filled: 0,
        }
    }

    fn reserve(&mut self, new_len: usize) {
        if self.buf.len() < new_len {
            self.buf.resize(new_len, 0);
        }
    }

    fn spare_capacity_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.filled..]
    }

    fn read(&mut self, r: impl FnOnce(&mut [u8]) -> Result<usize>) -> Result<()> {
        let read = r(self.spare_capacity_mut())?;
        self.filled += read;
        debug_assert!(self.filled <= self.buf.len());
        Ok(())
    }

    fn len(&self) -> usize {
        self.filled
    }

    fn take(&mut self) -> Vec<u8> {
        self.buf.truncate(self.filled);
        self.filled = 0;
        std::mem::take(&mut self.buf)
    }

    fn peek(&self) -> &[u8] {
        &self.buf[..self.filled]
    }
}

impl RecvConn {
    #[deprecated = "use poll() or select() on the file descriptor"]
    pub fn can_read_from_source(&self) -> io::Result<bool> {
        let mut fdset = nix::sys::select::FdSet::new();
        fdset.insert(self.stream.as_fd());

        use nix::sys::time::TimeValLike;
        let mut zero_timeout = nix::sys::time::TimeVal::microseconds(0);

        nix::sys::select::select(None, Some(&mut fdset), None, None, Some(&mut zero_timeout))?;
        Ok(fdset.contains(self.stream.as_fd()))
    }

    /// Reads from the source once but takes care that the internal buffer only reaches at maximum max_buffer_size
    /// so we can process messages separatly and avoid leaking file descriptors to wrong messages
    fn refill_buffer(&mut self, max_buffer_size: usize, timeout: Timeout) -> Result<()> {
        self.msg_buf_in.reserve(max_buffer_size);

        // Borrow all the fields because we can't use self in the closure...
        let cmsgspace = &mut self.cmsgspace;
        cmsgspace.clear();
        let fds_in = &mut self.fds_in;
        let stream = &mut self.stream;

        self.msg_buf_in.read(|buffer| {
            let iovec = IoSliceMut::new(buffer);

            let flags = MsgFlags::empty();

            let old_timeout = stream.read_timeout()?;
            match timeout {
                Timeout::Duration(d) => {
                    stream.set_read_timeout(Some(d))?;
                }
                Timeout::Infinite => {
                    stream.set_read_timeout(None)?;
                }
                Timeout::Nonblock => {
                    stream.set_nonblocking(true)?;
                }
            }
            let iovec_mut = &mut [iovec];
            let msg =
                recvmsg::<SockaddrStorage>(stream.as_raw_fd(), iovec_mut, Some(cmsgspace), flags)
                    .map_err(|e| match e {
                        nix::errno::Errno::EAGAIN => Error::TimedOut,
                        _ => Error::IoError(e.into()),
                    });

            stream.set_nonblocking(false)?;
            stream.set_read_timeout(old_timeout)?;

            let msg = msg?;

            if msg.bytes == 0 {
                return Err(Error::ConnectionClosed);
            }

            for cmsg in msg.cmsgs() {
                match cmsg {
                    ControlMessageOwned::ScmRights(fds) => {
                        fds_in.extend(fds.into_iter().map(UnixFd::new));
                    }
                    _ => {
                        // TODO what to do?
                        eprintln!("Cmsg other than ScmRights: {:?}", cmsg);
                    }
                }
            }

            Ok(msg.bytes)
        })?;

        Ok(())
    }

    pub fn bytes_needed_for_current_message(&self) -> Result<usize> {
        if self.msg_buf_in.len() < 16 {
            return Ok(16);
        }
        let msg_buf_in = &self.msg_buf_in.peek();
        let (_, header) = unmarshal::unmarshal_header(msg_buf_in, 0)?;
        let (_, header_fields_len) =
            crate::wire::util::parse_u32(&msg_buf_in[unmarshal::HEADER_LEN..], header.byteorder)?;
        let complete_header_size = unmarshal::HEADER_LEN + header_fields_len as usize + 4; // +4 because the length of the header fields does not count

        let padding_between_header_and_body = 8 - ((complete_header_size) % 8);
        let padding_between_header_and_body = if padding_between_header_and_body == 8 {
            0
        } else {
            padding_between_header_and_body
        };

        let bytes_needed =
            complete_header_size + padding_between_header_and_body + header.body_len as usize;
        Ok(bytes_needed)
    }

    // Checks if the internal buffer currently holds a complete message
    pub fn buffer_contains_whole_message(&self) -> Result<bool> {
        if self.msg_buf_in.len() < 16 {
            return Ok(false);
        }
        let bytes_needed = self.bytes_needed_for_current_message();
        match bytes_needed {
            Err(e) => {
                if let Error::UnmarshalError(UnmarshalError::NotEnoughBytes) = e {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
            Ok(bytes_needed) => Ok(self.msg_buf_in.len() >= bytes_needed),
        }
    }
    /// Blocks until a message has been read from the conn or the timeout has been reached
    pub fn read_whole_message(&mut self, timeout: Timeout) -> Result<()> {
        // This whole dance around reading exact amounts of bytes is necessary to read messages exactly at their bounds.
        // I think thats necessary so we can later add support for unixfd sending
        //calc timeout in reference to this point in time
        let start_time = time::Instant::now();

        while !self.buffer_contains_whole_message()? {
            self.refill_buffer(
                self.bytes_needed_for_current_message()?,
                super::calc_timeout_left(&start_time, timeout)?,
            )?;
        }
        Ok(())
    }

    /// Blocks until one read towards the message has been performed from the conn or the timeout has been reached
    pub fn read_once(&mut self, timeout: Timeout) -> Result<()> {
        self.refill_buffer(self.bytes_needed_for_current_message()?, timeout)?;
        Ok(())
    }

    /// Blocks until a message has been read from the conn or the timeout has been reached
    pub fn get_next_message(&mut self, timeout: Timeout) -> Result<MarshalledMessage> {
        self.read_whole_message(timeout)?;
        let (hdrbytes, header) = unmarshal::unmarshal_header(self.msg_buf_in.peek(), 0)?;
        let (dynhdrbytes, dynheader) =
            unmarshal::unmarshal_dynamic_header(&header, self.msg_buf_in.peek(), hdrbytes)?;

        let buf = self.msg_buf_in.take();
        let buf_len = buf.len();
        let (bytes_used, mut msg) =
            unmarshal::unmarshal_next_message(&header, dynheader, buf, hdrbytes + dynhdrbytes)?;

        msg.body.raw_fds = std::mem::take(&mut self.fds_in);

        if buf_len != bytes_used + hdrbytes + dynhdrbytes {
            return Err(Error::UnmarshalError(UnmarshalError::NotAllBytesUsed));
        }

        Ok(msg)
    }
}

impl SendConn {
    /// get the next new serial
    pub fn alloc_serial(&mut self) -> u32 {
        let serial = self.serial_counter;
        self.serial_counter += 1;
        serial
    }

    /// send a message over the conn
    pub fn send_message<'a>(
        &'a mut self,
        msg: &'a MarshalledMessage,
    ) -> Result<SendMessageContext<'a>> {
        let serial = if let Some(serial) = msg.dynheader.serial {
            serial
        } else {
            let serial = self.serial_counter;
            self.serial_counter += 1;
            serial
        };

        // clear the buf before marshalling the new header
        self.header_buf.clear();
        marshal::marshal(msg, serial, &mut self.header_buf)?;

        let ctx = SendMessageContext {
            msg,
            conn: self,

            state: SendMessageState {
                bytes_sent: 0,
                serial,
            },
        };

        Ok(ctx)
    }

    /// send a message and block until all bytes have been sent. Returns the serial of the message to match the response.
    pub fn send_message_write_all(&mut self, msg: &MarshalledMessage) -> Result<u32> {
        let ctx = self.send_message(msg)?;
        ctx.write_all().map_err(force_finish_on_error)
    }
}

/// only call if you deem the connection doomed by an error returned from writing.
/// The connection might be left in an invalid state if some but not all bytes of the message
/// have been written
pub fn force_finish_on_error<E>((s, e): (SendMessageContext<'_>, E)) -> E {
    s.force_finish();
    e
}

#[must_use = "Dropping this type is considered an error since it might leave the connection in an illdefined state if only some bytes of a message have been written"]
#[derive(Debug)]
/// Handles the process of actually sending a message over the connection it was created from. This allows graceful handling of short writes or timeouts with only
/// parts of the message written. You can loop over write or write_once or use write_all to wait until all bytes have been written or an error besides a timeout
/// arises.
pub struct SendMessageContext<'a> {
    msg: &'a MarshalledMessage,
    conn: &'a mut SendConn,

    state: SendMessageState,
}

/// Tracks the progress of sending the message. Can be used to resume a SendMessageContext.
///
///Note that this only makes sense if you resume with the same Message and Connection that were used to create the original SendMessageContext.
#[derive(Debug, Copy, Clone)]
pub struct SendMessageState {
    bytes_sent: usize,
    serial: u32,
}

/// This panics if the SendMessageContext was dropped when it was not yet finished. Use force_finish / force_finish_on_error
/// if you want to do this. It will be necessary for handling errors that make the connection unusable.
impl Drop for SendMessageContext<'_> {
    fn drop(&mut self) {
        if self.state.bytes_sent != 0 && !self.all_bytes_written() {
            panic!("You dropped a SendMessageContext that only partially sent the message! This is not ok since that leaves the connection in an ill defined state. Use one of the consuming functions!");
        } else {
            // No special cleanup needed
        }
    }
}

impl SendMessageContext<'_> {
    pub fn serial(&self) -> u32 {
        self.state.serial
    }

    /// Resume a SendMessageContext from the progress. This needs to be called with the same
    /// conn and msg that were used to create the original SendMessageContext.
    pub fn resume<'a>(
        conn: &'a mut SendConn,
        msg: &'a MarshalledMessage,
        progress: SendMessageState,
    ) -> SendMessageContext<'a> {
        SendMessageContext {
            conn,
            msg,
            state: progress,
        }
    }

    /// Turn this into the progress to resume the sending later. Note that you cannot send another
    /// message while doing that. You need to resume a SendMessageContext from this progress and
    /// send the current message beofre starting the next one.
    pub fn into_progress(self) -> SendMessageState {
        let progress = self.state;
        Self::force_finish(self);
        progress
    }

    /// Either drop self and return Ok value or return (self, error)
    fn finish_if_ok<O, E>(
        self,
        res: std::result::Result<O, E>,
    ) -> std::result::Result<O, (Self, E)> {
        match res {
            Ok(o) => {
                // this is technically unnecessary but just to make it explicit we drop self here
                std::mem::drop(self);
                Ok(o)
            }
            Err(e) => Err((self, e)),
        }
    }

    /// only call if you deem the connection doomed by an error returned from writing.
    /// The connection might be left in an invalid state if some but not all bytes of the message
    /// have been written
    pub fn force_finish(self) {
        std::mem::forget(self)
    }

    /// Try writing as many bytes as possible until either no more bytes need to be written or
    /// the timeout is reached. For an infinite timeout there is write_all as a shortcut
    pub fn write(mut self, timeout: Timeout) -> std::result::Result<u32, (Self, super::Error)> {
        let start_time = std::time::Instant::now();

        // loop until either the time is up or all bytes have been written
        let res = loop {
            let iteration_timeout = super::calc_timeout_left(&start_time, timeout);
            let iteration_timeout = match iteration_timeout {
                Err(e) => break Err(e),
                Ok(t) => t,
            };

            match self.write_once(iteration_timeout) {
                Err(e) => break Err(e),
                Ok(t) => t,
            };
            if self.all_bytes_written() {
                break Ok(self.state.serial);
            }
        };

        // This only occurs if all bytes have been sent. Otherwise we return with Error::TimedOut or another error
        self.finish_if_ok(res)
    }

    /// Block until all bytes have been written
    pub fn write_all(self) -> std::result::Result<u32, (Self, super::Error)> {
        self.write(Timeout::Infinite)
    }

    /// How many bytes need to be sent in total
    pub fn bytes_total(&self) -> usize {
        self.conn.header_buf.len() + self.msg.get_buf().len()
    }

    /// Check if all bytes have been written
    pub fn all_bytes_written(&self) -> bool {
        self.state.bytes_sent == self.bytes_total()
    }

    /// Basic routine to do a write to the fd once. Mostly useful if you are using a nonblocking timeout. But even then I would recommend using
    /// write() and not write_once()
    pub fn write_once(&mut self, timeout: Timeout) -> Result<usize> {
        // This will result in a zero sized slice if the header has been sent. Actually we would not need to
        // include that anymore in the iov but that is harder than just giving it the zero sized slice.
        let header_bytes_sent = usize::min(self.state.bytes_sent, self.conn.header_buf.len());
        let header_slice_to_send = &self.conn.header_buf[header_bytes_sent..];

        let body_bytes_sent = self.state.bytes_sent - header_bytes_sent;
        let body_slice_to_send = &self.msg.get_buf()[body_bytes_sent..];

        let iov = [
            IoSlice::new(header_slice_to_send),
            IoSlice::new(body_slice_to_send),
        ];
        let flags = MsgFlags::empty();

        let old_timeout = self.conn.stream.write_timeout()?;
        match timeout {
            Timeout::Duration(d) => {
                self.conn.stream.set_write_timeout(Some(d))?;
            }
            Timeout::Infinite => {
                self.conn.stream.set_write_timeout(None)?;
            }
            Timeout::Nonblock => {
                self.conn.stream.set_nonblocking(true)?;
            }
        }

        // if this is not the first write for this message do not send the raw_fds again. This would lead to unexpected
        // duplicated FDs on the other end!
        let raw_fds = if self.state.bytes_sent == 0 {
            self.msg
                .body
                .raw_fds
                .iter()
                .filter_map(|fd| fd.get_raw_fd())
                .collect::<Vec<RawFd>>()
        } else {
            vec![]
        };
        let bytes_sent = sendmsg::<SockaddrStorage>(
            self.conn.stream.as_raw_fd(),
            &iov,
            &[ControlMessage::ScmRights(&raw_fds)],
            flags,
            None,
        );

        self.conn.stream.set_write_timeout(old_timeout)?;
        self.conn.stream.set_nonblocking(false)?;

        let bytes_sent = bytes_sent.map_err(io::Error::from)?;

        self.state.bytes_sent += bytes_sent;

        Ok(bytes_sent)
    }
}

impl DuplexConn {
    /// Connect to a unix socket
    ///
    /// Remember to send the mandatory hello message before doing anything else with the connection!
    /// You can use the `send_hello` function for this.
    pub fn connect_to_bus(addr: UnixAddr, with_unix_fd: bool) -> super::Result<DuplexConn> {
        let sock = socket(
            socket::AddressFamily::Unix,
            socket::SockType::Stream,
            socket::SockFlag::empty(),
            None,
        )
        .map_err(io::Error::from)?;

        connect(sock.as_raw_fd(), &addr).map_err(io::Error::from)?;
        let mut stream = UnixStream::from(sock);
        match auth::do_auth(&mut stream)? {
            auth::AuthResult::Ok => {}
            auth::AuthResult::Rejected => return Err(Error::AuthFailed),
        }

        if with_unix_fd {
            match auth::negotiate_unix_fds(&mut stream)? {
                auth::AuthResult::Ok => {}
                auth::AuthResult::Rejected => return Err(Error::UnixFdNegotiationFailed),
            }
        }

        auth::send_begin(&mut stream)?;

        Ok(DuplexConn {
            send: SendConn {
                stream: stream.try_clone()?,
                header_buf: Vec::new(),
                serial_counter: 1,
            },
            recv: RecvConn {
                msg_buf_in: IncomingBuffer::new(),
                fds_in: Vec::new(),
                cmsgspace: cmsg_space!([RawFd; 10]),
                stream,
            },
        })
    }

    /// Sends the obligatory hello message and returns the unique id the daemon assigned this connection
    pub fn send_hello(&mut self, timeout: crate::connection::Timeout) -> super::Result<String> {
        let start_time = time::Instant::now();

        let hello = crate::standard_messages::hello();
        let serial = self
            .send
            .send_message(&hello)?
            .write(super::calc_timeout_left(&start_time, timeout)?)
            .map_err(|(ctx, e)| {
                ctx.force_finish();
                e
            })?;
        let resp = self
            .recv
            .get_next_message(super::calc_timeout_left(&start_time, timeout)?)?;
        if resp.dynheader.response_serial != Some(serial) {
            return Err(super::Error::AuthFailed);
        }
        let unique_name = resp.body.parser().get::<String>()?;
        Ok(unique_name)
    }
}

impl AsRawFd for SendConn {
    /// Reading or writing to the `RawFd` may result in undefined behavior
    /// and break the `Conn`.
    fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

impl AsRawFd for RecvConn {
    /// Reading or writing to the `RawFd` may result in undefined behavior
    /// and break the `Conn`.
    fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

impl AsRawFd for DuplexConn {
    /// Reading or writing to the `RawFd` may result in undefined behavior
    /// and break the `Conn`.
    fn as_raw_fd(&self) -> RawFd {
        self.recv.stream.as_raw_fd()
    }
}
