use crate::auth;
use crate::marshal;
use crate::message;
use crate::unmarshal;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use nix::sys::socket::recvmsg;
use nix::sys::socket::ControlMessageOwned;
use nix::sys::socket::MsgFlags;
use nix::sys::uio::IoVec;

#[derive(Debug)]
pub struct Conn {
    socket_path: PathBuf,
    stream: UnixStream,

    byteorder: message::ByteOrder,

    msg_buf_in: Vec<u8>,
    msg_buf_out: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    NixError(nix::Error),
    UnmarshalError(unmarshal::Error),
    MarshalError(message::Error),
    AuthFailed,
    UnixFdNegotiationFailed,
    NameTaken,
    AddressTypeNotSupported(String),
    PathDoesNotExist(String),
    NoAdressFound,
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}

impl std::convert::From<unmarshal::Error> for Error {
    fn from(e: unmarshal::Error) -> Error {
        Error::UnmarshalError(e)
    }
}
impl std::convert::From<message::Error> for Error {
    fn from(e: message::Error) -> Error {
        Error::MarshalError(e)
    }
}
impl std::convert::From<nix::Error> for Error {
    fn from(e: nix::Error) -> Error {
        Error::NixError(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

impl Conn {
    pub fn connect_to_bus_with_byteorder(
        path: PathBuf,
        byteorder: message::ByteOrder,
    ) -> Result<Conn> {
        let mut stream = UnixStream::connect(&path)?;
        match auth::do_auth(&mut stream)? {
            auth::AuthResult::Ok => {}
            auth::AuthResult::Rejected => return Err(Error::AuthFailed),
        }

        match auth::negotiate_unix_fds(&mut stream)? {
            auth::AuthResult::Ok => {}
            auth::AuthResult::Rejected => return Err(Error::UnixFdNegotiationFailed),
        }

        auth::send_begin(&mut stream)?;

        Ok(Conn {
            socket_path: path,
            stream,
            msg_buf_in: Vec::new(),
            msg_buf_out: Vec::new(),
            byteorder,
        })
    }
    pub fn connect_to_bus(path: PathBuf) -> Result<Conn> {
        Self::connect_to_bus_with_byteorder(path, message::ByteOrder::LittleEndian)
    }

    pub fn request_name(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }

    fn refill_buffer(&mut self, max_buffer_size: usize) -> Result<Vec<ControlMessageOwned>> {
        println!("Refill to: {}", max_buffer_size);
        let bytes_to_read = max_buffer_size - self.msg_buf_in.len();

        const BUFSIZE: usize = 512;
        let mut tmpbuf = [0u8; BUFSIZE];
        let iovec = IoVec::from_mut_slice(&mut tmpbuf[..usize::min(bytes_to_read, BUFSIZE)]);

        let mut cmsgspace = cmsg_space!([RawFd; 10]);
        let flags = MsgFlags::empty();

        let msg = recvmsg(
            self.stream.as_raw_fd(),
            &[iovec],
            Some(&mut cmsgspace),
            flags,
        )?;
        let cmsgs = msg.cmsgs().collect();

        self.msg_buf_in
            .extend(&mut tmpbuf[..msg.bytes].iter().copied());
        Ok(cmsgs)
    }

    pub fn get_next_message(&mut self) -> Result<message::Message> {
        // This whole dance around reading exact amounts of bytes is necessary to read messages exactly at their bounds.
        // I think thats necessary so we can later add support for unixfd sending
        let mut cmsgs = Vec::new();

        let header = loop {
            match unmarshal::unmarshal_header(&mut self.msg_buf_in) {
                Ok(header) => break header,
                Err(unmarshal::Error::NotEnoughBytes) => {}
                Err(e) => return Err(Error::from(e)),
            }
            let new_cmsgs = self.refill_buffer(unmarshal::HEADER_LEN)?;
            cmsgs.extend(new_cmsgs);
        };
        println!("Got header: {:?}", header);

        let mut header_fields_len = [0u8; 4];
        self.stream.read_exact(&mut header_fields_len[..])?;
        let header_fields_len =
            unmarshal::read_u32(&mut header_fields_len.to_vec(), header.byteorder)?;
        println!("Header fields bytes: {}", header_fields_len);
        marshal::write_u32(header_fields_len, header.byteorder, &mut self.msg_buf_in);

        let complete_header_size = unmarshal::HEADER_LEN + header_fields_len as usize + 4; // +4 because the length of the header fields does not count

        let padding_between_header_and_body = 8 - ((complete_header_size) % 8);
        let padding_between_header_and_body = if padding_between_header_and_body == 8 {
            0
        } else {
            padding_between_header_and_body
        };
        println!(
            "Bytes padding header <-> body {}, (because complete header size is: {})",
            padding_between_header_and_body, complete_header_size
        );

        let bytes_needed =
            (header.body_len + header_fields_len + 4) as usize + padding_between_header_and_body; // +4 because the length of the header fields does not count
        loop {
            println!("Buf size before read: {}", self.msg_buf_in.len());
            let new_cmsgs = self.refill_buffer(bytes_needed)?;
            cmsgs.extend(new_cmsgs);
            println!("Buf size after read: {}", self.msg_buf_in.len());
            if self.msg_buf_in.len() == bytes_needed {
                break;
            }
        }
        let mut msg = unmarshal::unmarshal_next_message(&header, &mut self.msg_buf_in)?;

        for cmsg in cmsgs {
            match cmsg {
                ControlMessageOwned::ScmRights(fds) => {
                    msg.raw_fds.extend(fds);
                }
                _ => {
                    // TODO what to do?
                    println!("Cmsg other than ScmRights: {:?}", cmsg);
                }
            }
        }
        Ok(msg)
    }

    pub fn send_message(&mut self, msg: &message::Message) -> Result<()> {
        self.msg_buf_out.clear();
        marshal::marshal(
            msg,
            message::ByteOrder::LittleEndian,
            &vec![],
            &mut self.msg_buf_out,
        )?;
        println!("Message: {:?}", self.msg_buf_out);
        //let mut clone_msg = buf.clone();
        //let msg_header = unmarshal::unmarshal_header(&mut clone_msg).unwrap();
        //println!("unmarshaled header: {:?}", msg_header);
        //let msg = unmarshal::unmarshal_next_message(&msg_header, &mut clone_msg).unwrap();

        self.stream.write_all(&self.msg_buf_out)?;
        println!("Written {} bytes", self.msg_buf_out.len());
        Ok(())
    }
}

pub fn get_session_bus_path() -> Result<PathBuf> {
    if let Ok(envvar) = std::env::var("DBUS_SESSION_BUS_ADDRESS") {
        if envvar.starts_with("unix:path=") {
            let ps = envvar.trim_start_matches("unix:path=");
            let p = PathBuf::from(&ps);
            if p.exists() {
                Ok(p)
            } else {
                Err(Error::PathDoesNotExist(ps.to_owned()))
            }
        } else {
            Err(Error::AddressTypeNotSupported(envvar))
        }
    } else {
        Err(Error::NoAdressFound)
    }
}
pub fn get_system_bus_path() -> Result<PathBuf> {
    let ps = "/run/dbus/system_bus_socket";
    let p = PathBuf::from(&ps);
    if p.exists() {
        Ok(p)
    } else {
        Err(Error::PathDoesNotExist(ps.to_owned()))
    }
}
