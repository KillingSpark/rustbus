//! Everything that deals with converting from/to raw bytes. You probably do not need this.

pub mod marshal;
pub mod unmarshal;
pub mod util;
pub mod validate_raw;

/// The different header fields a message may or maynot have
#[derive(Debug)]
pub enum HeaderField {
    Path(String),
    Interface(String),
    Member(String),
    ErrorName(String),
    ReplySerial(u32),
    Destination(String),
    Sender(String),
    Signature(String),
    UnixFds(u32),
}
