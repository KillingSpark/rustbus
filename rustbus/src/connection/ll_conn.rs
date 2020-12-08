use super::Error;
use super::Result;
use super::Timeout;
use crate::auth;
use crate::message_builder::MarshalledMessage;
use crate::wire::marshal;
use crate::wire::unmarshal;

use std::time;

use std::os::unix::io::RawFd;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;

use nix::cmsg_space;
use nix::sys::socket::{
    self, connect, recvmsg, sendmsg, socket, ControlMessage, ControlMessageOwned, MsgFlags,
    SockAddr, UnixAddr,
};
use nix::sys::uio::IoVec;

/// A lowlevel abstraction over the raw unix socket
#[derive(Debug)]
pub struct SendConn {
    stream: UnixStream,

    msg_buf_out: Vec<u8>,

    serial_counter: u32,
}

pub struct RecvConn {
    stream: UnixStream,

    msg_buf_in: Vec<u8>,
    cmsgs_in: Vec<ControlMessageOwned>,
}

pub struct DuplexConn {
    pub send: SendConn,
    pub recv: RecvConn,
}

impl RecvConn {
    pub fn can_read_from_source(&self) -> nix::Result<bool> {
        let mut fdset = nix::sys::select::FdSet::new();
        let fd = self.stream.as_raw_fd();
        fdset.insert(fd);

        use nix::sys::time::TimeValLike;
        let mut zero_timeout = nix::sys::time::TimeVal::microseconds(0);

        nix::sys::select::select(None, Some(&mut fdset), None, None, Some(&mut zero_timeout))?;
        Ok(fdset.contains(fd))
    }

    /// Reads from the source once but takes care that the internal buffer only reaches at maximum max_buffer_size
    /// so we can process messages separatly and avoid leaking file descriptors to wrong messages
    fn refill_buffer(&mut self, max_buffer_size: usize, timeout: Timeout) -> Result<()> {
        let bytes_to_read = max_buffer_size - self.msg_buf_in.len();

        const BUFSIZE: usize = 512;
        let mut tmpbuf = [0u8; BUFSIZE];
        let iovec = IoVec::from_mut_slice(&mut tmpbuf[..usize::min(bytes_to_read, BUFSIZE)]);

        let mut cmsgspace = cmsg_space!([RawFd; 10]);
        let flags = MsgFlags::empty();

        let old_timeout = self.stream.read_timeout()?;
        match timeout {
            Timeout::Duration(d) => {
                self.stream.set_read_timeout(Some(d))?;
            }
            Timeout::Infinite => {
                self.stream.set_read_timeout(None)?;
            }
            Timeout::Nonblock => {
                self.stream.set_nonblocking(true)?;
            }
        }
        let msg = recvmsg(
            self.stream.as_raw_fd(),
            &[iovec],
            Some(&mut cmsgspace),
            flags,
        )
        .map_err(|e| match e.as_errno() {
            Some(nix::errno::Errno::EAGAIN) => Error::TimedOut,
            _ => Error::NixError(e),
        });

        self.stream.set_nonblocking(false)?;
        self.stream.set_read_timeout(old_timeout)?;

        let msg = msg?;

        self.msg_buf_in
            .extend(&mut tmpbuf[..msg.bytes].iter().copied());
        self.cmsgs_in.extend(msg.cmsgs());
        Ok(())
    }

    pub fn bytes_needed_for_current_message(&self) -> Result<usize> {
        if self.msg_buf_in.len() < 16 {
            return Ok(16);
        }
        let (_, header) = unmarshal::unmarshal_header(&self.msg_buf_in, 0)?;
        let (_, header_fields_len) = crate::wire::util::parse_u32(
            &self.msg_buf_in[unmarshal::HEADER_LEN..],
            header.byteorder,
        )?;
        let complete_header_size = unmarshal::HEADER_LEN + header_fields_len as usize + 4; // +4 because the length of the header fields does not count

        let padding_between_header_and_body = 8 - ((complete_header_size) % 8);
        let padding_between_header_and_body = if padding_between_header_and_body == 8 {
            0
        } else {
            padding_between_header_and_body
        };

        let bytes_needed = complete_header_size as usize
            + padding_between_header_and_body
            + header.body_len as usize;
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
                if let Error::UnmarshalError(unmarshal::Error::NotEnoughBytes) = e {
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
        let (hdrbytes, header) = unmarshal::unmarshal_header(&self.msg_buf_in, 0)?;
        let (dynhdrbytes, dynheader) =
            unmarshal::unmarshal_dynamic_header(&header, &self.msg_buf_in, hdrbytes)?;

        let (bytes_used, mut msg) = unmarshal::unmarshal_next_message(
            &header,
            dynheader,
            &self.msg_buf_in,
            hdrbytes + dynhdrbytes,
        )?;

        if self.msg_buf_in.len() != bytes_used + hdrbytes + dynhdrbytes {
            return Err(Error::UnmarshalError(unmarshal::Error::NotAllBytesUsed));
        }
        self.msg_buf_in.clear();

        for cmsg in &self.cmsgs_in {
            match cmsg {
                ControlMessageOwned::ScmRights(fds) => {
                    msg.body
                        .raw_fds
                        .extend(fds.iter().map(|fd| crate::wire::UnixFd::new(*fd)));
                }
                _ => {
                    // TODO what to do?
                    eprintln!("Cmsg other than ScmRights: {:?}", cmsg);
                }
            }
        }
        self.cmsgs_in.clear();

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
    pub fn send_message(
        &mut self,
        msg: &mut crate::message_builder::MarshalledMessage,
        timeout: Timeout,
    ) -> Result<u32> {
        self.msg_buf_out.clear();
        let (remove_later, serial) = if let Some(serial) = msg.dynheader.serial {
            (false, serial)
        } else {
            let serial = self.serial_counter;
            self.serial_counter += 1;
            msg.dynheader.serial = Some(serial);
            (true, serial)
        };

        marshal::marshal(&msg, msg.body.byteorder, &mut self.msg_buf_out)?;

        let iov = [IoVec::from_slice(&self.msg_buf_out)];
        let flags = MsgFlags::empty();

        let old_timeout = self.stream.write_timeout()?;
        match timeout {
            Timeout::Duration(d) => {
                self.stream.set_write_timeout(Some(d))?;
            }
            Timeout::Infinite => {
                self.stream.set_write_timeout(None)?;
            }
            Timeout::Nonblock => {
                self.stream.set_nonblocking(true)?;
            }
        }
        let raw_fds = msg
            .body
            .raw_fds
            .iter()
            .map(|fd| fd.get_raw_fd())
            .flatten()
            .collect::<Vec<RawFd>>();
        let l = sendmsg(
            self.stream.as_raw_fd(),
            &iov,
            &[ControlMessage::ScmRights(&raw_fds)],
            flags,
            None,
        );

        self.stream.set_write_timeout(old_timeout)?;
        self.stream.set_nonblocking(false)?;

        let l = l?;

        assert_eq!(l, self.msg_buf_out.len());

        if remove_later {
            msg.dynheader.serial = None;
        }

        Ok(serial)
    }
}

impl DuplexConn {
    /// Connect to a unix socket and choose a byteorder
    pub fn connect_to_bus(addr: UnixAddr, with_unix_fd: bool) -> super::Result<DuplexConn> {
        let sock = socket(
            socket::AddressFamily::Unix,
            socket::SockType::Stream,
            socket::SockFlag::empty(),
            None,
        )?;
        let sock_addr = SockAddr::Unix(addr);
        connect(sock, &sock_addr)?;
        let mut stream = unsafe { UnixStream::from_raw_fd(sock) };
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
                msg_buf_out: Vec::new(),
                serial_counter: 1,
            },
            recv: RecvConn {
                msg_buf_in: Vec::new(),
                cmsgs_in: Vec::new(),
                stream,
            },
        })
    }

    /// Sends the obligatory hello message and returns the unique id the daemon assigned this connection
    pub fn send_hello(&mut self, timeout: crate::connection::Timeout) -> super::Result<String> {
        let start_time = time::Instant::now();

        let serial = self.send.send_message(
            &mut crate::standard_messages::hello(),
            super::calc_timeout_left(&start_time, timeout)?,
        )?;
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
