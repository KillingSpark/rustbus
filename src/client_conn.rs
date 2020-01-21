use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Conn {
    socket_path: PathBuf,
    stream: UnixStream,
}

pub enum Error {
    IoError(std::io::Error),
    NameTaken
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

impl Conn {
    pub fn connect_to_bus(path: PathBuf) -> Result<Conn> {
        let stream = UnixStream::connect(&path)?;

        Ok(Conn {
            socket_path: path,
            stream,
        })
    }

    pub fn request_name(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }
}
