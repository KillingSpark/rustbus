use crate::signature;

#[derive(Copy, Clone)]
pub enum MessageType {
    Signal,
    Error,
    Call,
    Reply,
}

pub enum Base {
    Int32(i32),
    Uint32(u32),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),
}

pub enum Container {
    Array(Vec<Param>),
    Struct(Vec<Param>),
    DictEntry(Base, Box<Param>),
    Variant(Box<Variant>),
}

pub struct Variant {
    pub sig: signature::Type,
    pub value: Param
}

pub enum Param {
    Base(Base),
    Container(Container),
}

pub struct Message {
    pub typ: MessageType,
    pub interface: Option<String>,
    pub member: Option<String>,
    pub params: Vec<Param>,
}

pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
    InvalidHeaderFields,
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

pub fn validate_header_fields(
    msg_type: MessageType,
    header_fields: &Vec<HeaderField>,
) -> Result<()> {
    let mut have_path = false;
    let mut have_interface = false;
    let mut have_member = false;
    let mut have_errorname = false;
    let mut have_replyserial = false;
    let mut have_destination = false;
    let mut have_sender = false;
    let mut have_signature = false;
    let mut have_unixfds = false;

    for h in header_fields {
        match h {
            HeaderField::Destination(_) => {
                if have_destination {
                    return Err(Error::InvalidHeaderFields);
                }
                have_destination = true;
            }
            HeaderField::ErrorName(_) => {
                if have_errorname {
                    return Err(Error::InvalidHeaderFields);
                }
                have_errorname = true;
            }
            HeaderField::Interface(_) => {
                if have_interface {
                    return Err(Error::InvalidHeaderFields);
                }
                have_interface = true;
            }
            HeaderField::Member(_) => {
                if have_member {
                    return Err(Error::InvalidHeaderFields);
                }
                have_member = true;
            }
            HeaderField::Path(_) => {
                if have_path {
                    return Err(Error::InvalidHeaderFields);
                }
                have_path = true;
            }
            HeaderField::ReplySerial(_) => {
                if have_replyserial {
                    return Err(Error::InvalidHeaderFields);
                }
                have_replyserial = true;
            }
            HeaderField::Sender(_) => {
                if have_sender {
                    return Err(Error::InvalidHeaderFields);
                }
                have_sender = true;
            }
            HeaderField::Signature(_) => {
                if have_signature {
                    return Err(Error::InvalidHeaderFields);
                }
                have_signature = true;
            }
            HeaderField::UnixFds(_) => {
                if have_unixfds {
                    return Err(Error::InvalidHeaderFields);
                }
                have_unixfds = true;
            }
        }
    }

    let valid = match msg_type {
        MessageType::Call => {
            have_path && have_member
        }
        MessageType::Signal => {
            have_path && have_member && have_interface
        }
        MessageType::Reply => {
            have_replyserial
        }
        MessageType::Error => {
            have_errorname && have_replyserial
        }
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidHeaderFields)
    }
}
