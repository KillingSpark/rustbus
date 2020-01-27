use crate::signature;
use std::os::unix::io::RawFd;

#[derive(Copy, Clone, Debug)]
pub enum MessageType {
    Signal,
    Error,
    Call,
    Reply,
    Invalid,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Base {
    Double(u64),
    Byte(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    UnixFd(u32),
    Int64(i64),
    Uint64(u64),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),
}

#[derive(Debug)]
pub enum Container {
    Array(Vec<Param>),
    Struct(Vec<Param>),
    Dict(std::collections::HashMap<Base, Param>),
    Variant(Box<Variant>),
}

#[derive(Debug)]
pub struct Variant {
    pub sig: signature::Type,
    pub value: Param,
}

#[derive(Debug)]
pub enum Param {
    Base(Base),
    Container(Container),
}

#[derive(Debug)]
pub struct Message {
    pub typ: MessageType,
    pub interface: Option<String>,
    pub member: Option<String>,
    pub object: Option<String>,
    pub destination: Option<String>,
    pub params: Vec<Param>,
    pub serial: Option<u32>,
    pub response_serial: Option<u32>,
    pub sender: Option<String>,
    pub error_name: Option<String>,

    pub num_fds: Option<u32>,
    pub raw_fds: Vec<RawFd>,
}

impl Message {
    pub fn new() -> Message {
        Message {
            typ: MessageType::Invalid,
            interface: None,
            member: None,
            params: Vec::new(),
            object: None,
            destination: None,
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            response_serial: None,
            sender: None,
            error_name: None,
        }
    }

    pub fn set_interface(&mut self, interface: String) {
        self.interface = Some(interface);
    }
    pub fn set_member(&mut self, member: String) {
        self.member = Some(member);
    }
    pub fn set_object(&mut self, object: String) {
        self.object = Some(object);
    }
    pub fn set_destination(&mut self, dest: String) {
        self.destination = Some(dest);
    }
    pub fn push_params(&mut self, params: Vec<Param>) {
        self.params.extend(params);
    }

    pub fn make_response(&self) -> Message {
        Message {
            typ: MessageType::Reply,
            interface: None,
            member: None,
            params: Vec::new(),
            object: None,
            destination: self.sender.clone(),
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            sender: None,
            response_serial: self.serial.clone(),
            error_name: None,
        }
    }
    pub fn make_error_response(&self, error_name: String) -> Message {
        Message {
            typ: MessageType::Reply,
            interface: None,
            member: None,
            params: Vec::new(),
            object: None,
            destination: self.sender.clone(),
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            sender: None,
            response_serial: self.serial.clone(),
            error_name: Some(error_name),
        }
    }
}

impl Param {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Param::Base(b) => b.make_signature(buf),
            Param::Container(c) => c.make_signature(buf),
        }
    }
}

impl Base {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Base::Boolean(_) => buf.push('c'),
            Base::Double(_) => buf.push('d'),
            Base::Byte(_) => buf.push('y'),
            Base::Int16(_) => buf.push('n'),
            Base::Uint16(_) => buf.push('q'),
            Base::Int32(_) => buf.push('i'),
            Base::Uint32(_) => buf.push('u'),
            Base::UnixFd(_) => buf.push('h'),
            Base::Int64(_) => buf.push('x'),
            Base::Uint64(_) => buf.push('t'),
            Base::ObjectPath(_) => buf.push('o'),
            Base::String(_) => buf.push('s'),
            Base::Signature(_) => buf.push('g'),
        }
    }
}
impl Container {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Container::Array(elements) => {
                buf.push('a');
                elements[0].make_signature(buf);
            }
            Container::Dict(map) => {
                let key = map.keys().next().unwrap();
                let val = map.get(key).unwrap();

                buf.push('{');
                key.make_signature(buf);
                val.make_signature(buf);
                buf.push('{');
            }
            Container::Struct(elements) => {
                buf.push('(');
                for el in elements {
                    el.make_signature(buf);
                }
                buf.push(')');
            }
            Container::Variant(_) => {
                buf.push('v');
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
    InvalidBusname,
    InvalidErrorname,
    InvalidMembername,
    InvalidInterface,
    InvalidHeaderFields,
    DuplicatedHeaderFields,
    NoSerial,
    InvalidType,
}

#[derive(Clone, Copy, Debug)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

pub enum HeaderFlags {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

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

pub type Result<T> = std::result::Result<T, Error>;

pub fn validate_object_path(op: &str) -> Result<()> {
    if op.is_empty() {
        return Err(Error::InvalidObjectPath);
    }
    if !op.starts_with('/') {
        return Err(Error::InvalidObjectPath);
    }
    if op.len() > 1 {
        let split = op.split('/').collect::<Vec<_>>();
        if split.len() < 2 {
            return Err(Error::InvalidObjectPath);
        }
        for element in &split[1..] {
            if element.is_empty() {
                return Err(Error::InvalidObjectPath);
            }
            if element.chars().nth(0).unwrap().is_numeric() {
                return Err(Error::InvalidObjectPath);
            }
            let alphanum_or_underscore = element
                .chars()
                .fold(true, |acc, c| acc && (c.is_alphanumeric() || c == '_'));
            if !alphanum_or_underscore {
                return Err(Error::InvalidObjectPath);
            }
        }
    }
    Ok(())
}
pub fn validate_interface(int: &str) -> Result<()> {
    if int.len() < 3 {
        return Err(Error::InvalidInterface);
    }
    if !int.contains('.') {
        return Err(Error::InvalidInterface);
    }

    let split = int.split('.').collect::<Vec<_>>();
    if split.len() < 2 {
        return Err(Error::InvalidInterface);
    }
    for element in split {
        if element.is_empty() {
            return Err(Error::InvalidInterface);
        }
        if element.chars().nth(0).unwrap().is_numeric() {
            return Err(Error::InvalidInterface);
        }
        let alphanum_or_underscore = element
            .chars()
            .fold(true, |acc, c| acc && (c.is_alphanumeric() || c == '_'));
        if !alphanum_or_underscore {
            return Err(Error::InvalidInterface);
        }
    }

    Ok(())
}

pub fn validate_errorname(en: &str) -> Result<()> {
    validate_interface(en).map_err(|_| Error::InvalidErrorname)
}

pub fn validate_busname(bn: &str) -> Result<()> {
    if bn.len() < 3 {
        return Err(Error::InvalidBusname);
    }
    if !bn.contains('.') {
        return Err(Error::InvalidBusname);
    }

    let split = bn.split('.').collect::<Vec<_>>();
    if split.len() < 2 {
        return Err(Error::InvalidBusname);
    }
    for idx in 0..split.len() {
        let element = split[idx];
        if element.is_empty() {
            return Err(Error::InvalidBusname);
        }
        if element.chars().nth(0).unwrap().is_numeric() {
            return Err(Error::InvalidBusname);
        }
        let alphanum_or_underscore_or_dash = element.chars().fold(true, |acc, c| {
            acc && (c.is_alphanumeric() || c == '_' || c == '-' || (idx == 0 && c == ':'))
        });
        if !alphanum_or_underscore_or_dash {
            return Err(Error::InvalidBusname);
        }
    }

    Ok(())
}

pub fn validate_membername(mem: &str) -> Result<()> {
    if mem.is_empty() {
        return Err(Error::InvalidBusname);
    }

    let alphanum_or_underscore = mem
        .chars()
        .fold(true, |acc, c| acc && (c.is_alphanumeric() || c == '_'));
    if !alphanum_or_underscore {
        return Err(Error::InvalidMembername);
    }

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
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_destination = true;
            }
            HeaderField::ErrorName(_) => {
                if have_errorname {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_errorname = true;
            }
            HeaderField::Interface(_) => {
                if have_interface {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_interface = true;
            }
            HeaderField::Member(_) => {
                if have_member {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_member = true;
            }
            HeaderField::Path(_) => {
                if have_path {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_path = true;
            }
            HeaderField::ReplySerial(_) => {
                if have_replyserial {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_replyserial = true;
            }
            HeaderField::Sender(_) => {
                if have_sender {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_sender = true;
            }
            HeaderField::Signature(_) => {
                if have_signature {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_signature = true;
            }
            HeaderField::UnixFds(_) => {
                if have_unixfds {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_unixfds = true;
            }
        }
    }

    let valid = match msg_type {
        MessageType::Invalid => false,
        MessageType::Call => have_path && have_member,
        MessageType::Signal => have_path && have_member && have_interface,
        MessageType::Reply => have_replyserial,
        MessageType::Error => have_errorname && have_replyserial,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidHeaderFields)
    }
}
