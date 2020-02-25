//! lowlevel message stuff

use crate::params::*;
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

/// A message with all the different fields it may or may not have
#[derive(Debug, Clone)]
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

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

impl Message {
    /// Create a new empty message
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

    /// Make a correctly addressed response with the correct response serial
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
            response_serial: self.serial,
            error_name: None,
        }
    }

    /// Make a correctly addressed error response with the correct response serial
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
            response_serial: self.serial,
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
    pub fn sig(&self) -> signature::Type {
        match self {
            Param::Base(b) => b.sig(),
            Param::Container(c) => c.sig(),
        }
    }
}

impl Base {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Base::Boolean(_) => buf.push('b'),
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

    pub fn sig(&self) -> signature::Type {
        let sig: signature::Base = self.into();
        signature::Type::Base(sig)
    }
}
impl Container {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Container::Array(elements) => {
                buf.push('a');
                elements.element_sig.to_str(buf);
            }
            Container::Dict(map) => {
                buf.push('a');
                buf.push('{');
                map.key_sig.to_str(buf);
                map.value_sig.to_str(buf);
                buf.push('}');
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

    pub fn sig(&self) -> signature::Type {
        let sig: signature::Container = self.into();
        signature::Type::Container(sig)
    }
}

/// The different errors that can occur when dealing with messages
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidObjectPath,
    InvalidSignature(signature::Error),
    InvalidSignatureTooManyTypes,
    InvalidSignatureShouldBeBase,
    InvalidBusname,
    InvalidErrorname,
    InvalidMembername,
    InvalidInterface,
    InvalidHeaderFields,
    DuplicatedHeaderFields,
    NoSerial,
    InvalidType,
    ArrayElementTypesDiffer,
    DictKeyTypesDiffer,
    DictValueTypesDiffer,
    EmptyArray,
    EmptyDict,
}

/// The supported byte orders
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

pub type Result<T> = std::result::Result<T, Error>;

//
//
// Param TO
//
//

impl std::convert::From<&Param> for signature::Type {
    fn from(p: &Param) -> crate::signature::Type {
        match p {
            Param::Base(b) => signature::Type::Base(b.into()),
            Param::Container(c) => signature::Type::Container(c.into()),
        }
    }
}

//
//
// Param FROM
//
//

impl std::convert::From<Base> for Param {
    fn from(s: Base) -> Self {
        Param::Base(s)
    }
}
impl std::convert::From<Container> for Param {
    fn from(s: Container) -> Self {
        Param::Container(s)
    }
}

impl std::convert::From<bool> for Param {
    fn from(s: bool) -> Self {
        Param::Base(Base::Boolean(s))
    }
}
impl std::convert::From<String> for Param {
    fn from(s: String) -> Self {
        Param::Base(Base::String(s))
    }
}
impl std::convert::From<u8> for Param {
    fn from(s: u8) -> Self {
        Param::Base(Base::Byte(s))
    }
}
impl std::convert::From<u16> for Param {
    fn from(s: u16) -> Self {
        Param::Base(Base::Uint16(s))
    }
}
impl std::convert::From<u32> for Param {
    fn from(s: u32) -> Self {
        Param::Base(Base::Uint32(s))
    }
}
impl std::convert::From<u64> for Param {
    fn from(s: u64) -> Self {
        Param::Base(Base::Uint64(s))
    }
}
impl std::convert::From<i16> for Param {
    fn from(s: i16) -> Self {
        Param::Base(Base::Int16(s))
    }
}
impl std::convert::From<i32> for Param {
    fn from(s: i32) -> Self {
        Param::Base(Base::Int32(s))
    }
}
impl std::convert::From<i64> for Param {
    fn from(s: i64) -> Self {
        Param::Base(Base::Int64(s))
    }
}

//
//
// Container FROM
//
//

impl std::convert::TryFrom<(signature::Type, Vec<Param>)> for Container {
    type Error = Error;
    fn try_from(parts: (signature::Type, Vec<Param>)) -> std::result::Result<Container, Error> {
        let arr = Array {
            element_sig: parts.0,
            values: parts.1,
        };
        validate_array(&arr)?;
        Ok(Container::Array(arr))
    }
}
impl std::convert::TryFrom<Vec<Param>> for Container {
    type Error = Error;
    fn try_from(elems: Vec<Param>) -> std::result::Result<Container, Error> {
        if elems.is_empty() {
            return Err(Error::EmptyArray);
        }
        Container::try_from((elems[0].sig(), elems))
    }
}

impl std::convert::TryFrom<(signature::Base, signature::Type, DictMap)> for Container {
    type Error = Error;
    fn try_from(
        parts: (signature::Base, signature::Type, DictMap),
    ) -> std::result::Result<Container, Error> {
        let dict = Dict {
            key_sig: parts.0,
            value_sig: parts.1,
            map: parts.2,
        };
        validate_dict(&dict)?;
        Ok(Container::Dict(dict))
    }
}
impl std::convert::TryFrom<DictMap> for Container {
    type Error = Error;
    fn try_from(elems: DictMap) -> std::result::Result<Container, Error> {
        if elems.is_empty() {
            return Err(Error::EmptyDict);
        }
        let key_sig = elems.keys().nth(0).unwrap().sig();
        let value_sig = elems.values().nth(0).unwrap().sig();

        if let signature::Type::Base(key_sig) = key_sig {
            Container::try_from((key_sig, value_sig, elems))
        } else {
            Err(Error::InvalidSignatureShouldBeBase)
        }
    }
}

//
//
// Base FROM
//
//

impl std::convert::From<bool> for Base {
    fn from(s: bool) -> Self {
        Base::Boolean(s)
    }
}
impl std::convert::From<String> for Base {
    fn from(s: String) -> Self {
        Base::String(s)
    }
}
impl std::convert::From<u8> for Base {
    fn from(s: u8) -> Self {
        Base::Byte(s)
    }
}
impl std::convert::From<u16> for Base {
    fn from(s: u16) -> Self {
        Base::Uint16(s)
    }
}
impl std::convert::From<u32> for Base {
    fn from(s: u32) -> Self {
        Base::Uint32(s)
    }
}
impl std::convert::From<u64> for Base {
    fn from(s: u64) -> Self {
        Base::Uint64(s)
    }
}
impl std::convert::From<i16> for Base {
    fn from(s: i16) -> Self {
        Base::Int16(s)
    }
}
impl std::convert::From<i32> for Base {
    fn from(s: i32) -> Self {
        Base::Int32(s)
    }
}
impl std::convert::From<i64> for Base {
    fn from(s: i64) -> Self {
        Base::Int64(s)
    }
}

//
//
// Container TO
//
//

impl std::convert::From<&Container> for signature::Container {
    fn from(c: &Container) -> crate::signature::Container {
        match c {
            Container::Array(arr) => signature::Container::Array(Box::new(arr.element_sig.clone())),
            Container::Dict(dict) => {
                signature::Container::Dict(dict.key_sig, Box::new(dict.value_sig.clone()))
            }
            Container::Struct(params) => {
                signature::Container::Struct(params.iter().map(|param| param.into()).collect())
            }
            Container::Variant(_) => signature::Container::Variant,
        }
    }
}
