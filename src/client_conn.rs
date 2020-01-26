use crate::auth;
use crate::marshal;
use crate::message;
use crate::unmarshal;
use std::io::Read;
use std::io::Write;
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
    MarshalError(message::Error),
    AuthFailed,
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
impl std::convert::From<message::Error> for Error {
    fn from(e: message::Error) -> Error {
        Error::MarshalError(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

impl Conn {
    pub fn connect_to_bus(path: PathBuf) -> Result<Conn> {
        let mut stream = UnixStream::connect(&path)?;
        match auth::do_auth(&mut stream)? {
            auth::AuthResult::Ok => Ok(Conn {
                socket_path: path,
                stream,
                msg_buf: Vec::new(),
            }),
            auth::AuthResult::Rejected => Err(Error::AuthFailed),
        }
    }

    pub fn request_name(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }

    fn refill_buffer(&mut self, max_buffer_size: usize) -> Result<()> {
        let mut tmpbuf = [0u8; 512];

        let bytes_to_read = max_buffer_size - self.msg_buf.len();
        let bytes = self.stream.read(&mut tmpbuf[..bytes_to_read])?;
        self.msg_buf.extend(&mut tmpbuf[..bytes].iter().copied());
        Ok(())
    }

    pub fn get_next_message(&mut self) -> Result<message::Message> {
        // This whole dance around reading exact amounts of bytes is necessary to read messages exactly at their bounds.
        // I think thats necessary so we can later add support for unixfd sending 


        let header = loop {
            match unmarshal::unmarshal_header(&mut self.msg_buf) {
                Ok(header) => break header,
                Err(unmarshal::Error::NotEnoughBytes) => {}
                Err(e) => return Err(Error::from(e)),
            }
        self.refill_buffer(unmarshal::HEADER_LEN)?;
        };
        println!("Got header: {:?}", header);

        let mut header_fields_len = [0u8;4];
        self.stream.read_exact(&mut header_fields_len[..])?;
        let header_fields_len = unmarshal::read_u32(&mut header_fields_len.to_vec(), header.byteorder)?;
        println!("Header fields bytes: {}", header_fields_len);
        marshal::write_u32(header_fields_len, header.byteorder, &mut self.msg_buf);

        let complete_header_size = unmarshal::HEADER_LEN + header_fields_len as usize + 4; // +4 because the length of the header fields does not count

        let padding_between_header_and_body = 8 - ((complete_header_size) % 8);
        println!("Bytes padding {}", padding_between_header_and_body);

        let bytes_needed = (header.body_len + header_fields_len + 4) as usize + padding_between_header_and_body; // +4 because the length of the header fields does not count
        loop {
            println!("Buf size before read: {}", self.msg_buf.len());
            self.refill_buffer(bytes_needed)?;
            println!("Buf size after read: {}", self.msg_buf.len());
            if self.msg_buf.len() == bytes_needed {
                break;
            }
        }
        let msg = unmarshal::unmarshal_next_message(&header, &mut self.msg_buf)?;
        Ok(msg)
    }

    pub fn send_message(&mut self, msg: &message::Message) -> Result<()> {
        let mut buf = Vec::new();
        marshal::marshal(msg, message::ByteOrder::LittleEndian, 1, &vec![], &mut buf)?;
        println!("Message: {:?}", buf); 
        
        //let mut clone_msg = buf.clone();
        //let msg_header = unmarshal::unmarshal_header(&mut clone_msg).unwrap();
        //println!("unmarshaled header: {:?}", msg_header);
        //let msg = unmarshal::unmarshal_next_message(&msg_header, &mut clone_msg).unwrap();

        self.stream.write_all(&buf)?;
        println!("Written {} bytes", buf.len());
        Ok(())
    }
}
