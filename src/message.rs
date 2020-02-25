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
pub struct Message<'a, 'e> {
    pub typ: MessageType,
    pub interface: Option<String>,
    pub member: Option<String>,
    pub object: Option<String>,
    pub destination: Option<String>,
    pub params: Vec<Param<'a, 'e>>,
    pub serial: Option<u32>,
    pub response_serial: Option<u32>,
    pub sender: Option<String>,
    pub error_name: Option<String>,

    pub num_fds: Option<u32>,
    pub raw_fds: Vec<RawFd>,
}

impl<'a, 'e> Default for Message<'a, 'e> {
    fn default() -> Message<'a, 'e> {
        Self::new()
    }
}

impl<'a, 'e> Message<'a, 'e> {
    /// Create a new empty message
    pub fn new() -> Message<'a, 'e> {
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
    pub fn push_params(&mut self, params: Vec<Param<'a, 'e>>) {
        self.params.extend(params);
    }
    pub fn push_param(&mut self, param: Param<'a, 'e>) {
        self.params.push(param);
    }

    /// Make a correctly addressed response with the correct response serial
    pub fn make_response(&self) -> Self {
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
    pub fn make_error_response(&self, error_name: String, error_msg: Option<String>) -> Self {
        let mut err_resp = Message {
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
        };
        if let Some(text) = error_msg {
            err_resp.push_param(text.into())
        }
        err_resp
    }

    pub fn sig(&self) -> Vec<signature::Type> {
        self.params.iter().map(|p| p.sig()).collect()
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
