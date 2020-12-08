//! Everything that deals with converting from/to raw bytes. You probably only need the various wrapper types.

pub mod marshal;
pub mod unixfd;
pub mod unmarshal;
pub mod util;
pub mod validate_raw;
pub mod variant_macros;

pub use unixfd::UnixFd;

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
