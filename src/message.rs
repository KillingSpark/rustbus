use crate::signature;

pub enum Message {
    Signal,
    Error,
    Call(Call),
    Reply,
}

pub enum Base {
    Int32(i32),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),
}

pub enum Container {
    Array(Vec<Param>),
    Struct(Vec<Param>),
    DictEntry(Base, Box<Param>)
}

pub enum Param {
    Base(Base),
    Container(Container),
}

pub struct Call {
    pub interface: String,
    pub member: String,
    pub params: Vec<Param>
}


pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
}

#[derive(Clone, Copy)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

pub enum HeaderFlags {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

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

pub type Result<T> = std::result::Result<T, Error>;

pub fn validate_object_path(_op: &str) -> Result<()> {
    // TODO
    Ok(())
}
pub fn validate_signature(sig: &str) -> Result<()> {
    if signature::Type::from_str(sig).is_err() {
        Err(Error::InvalidSignature)
    } else {
        Ok(())
    }
}

pub fn validate_array(_array: &Vec<Param>) -> Result<()> {
    // TODO check that all elements have the same type
    Ok(())
}