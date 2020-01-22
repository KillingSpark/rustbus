use crate::auth;
use crate::unmarshal;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Conn {
    socket_path: PathBuf,
    stream: UnixStream,

    msg_buf: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UnmarshalError(unmarshal::Error),
    NameTaken,
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

type Result<T> = std::result::Result<T, Error>;

impl Conn {
    pub fn connect_to_bus(path: PathBuf) -> Result<Conn> {
        let mut stream = UnixStream::connect(&path)?;
        auth::do_auth(&mut stream)?;

        Ok(Conn {
            socket_path: path,
            stream,
            msg_buf: Vec::new(),
        })
    }

    pub fn request_name(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }

    pub fn get_next_message(&mut self) -> Result<()> {
        let header = loop {
            match unmarshal::unmarshal_header(&mut self.msg_buf) {
                Ok(header) => break header,
                Err(unmarshal::Error::NotEnoughBytes) => {}
                Err(e) => return Err(Error::from(e)),
            }
        };
        Ok(())
    }
}
