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

#[derive(Debug, Clone, Default)]
pub struct DynamicHeader {
    pub interface: Option<String>,
    pub member: Option<String>,
    pub object: Option<String>,
    pub destination: Option<String>,
    pub serial: Option<u32>,
    pub sender: Option<String>,
    pub signature: Option<String>,
    pub error_name: Option<String>,
    pub response_serial: Option<u32>,
    pub num_fds: Option<u32>,
}

impl DynamicHeader {
    /// Make a correctly addressed error response with the correct response serial
    pub fn make_error_response(
        &self,
        error_name: String,
        error_msg: Option<String>,
    ) -> crate::message_builder::MarshalledMessage {
        let mut err_resp = crate::message_builder::MarshalledMessage {
            typ: MessageType::Reply,
            dynheader: DynamicHeader {
                interface: None,
                member: None,
                object: None,
                destination: self.sender.clone(),
                serial: None,
                num_fds: None,
                sender: None,
                signature: None,
                response_serial: self.serial,
                error_name: Some(error_name),
            },
            raw_fds: Vec::new(),
            flags: 0,
            body: crate::message_builder::MarshalledMessageBody::new(),
        };
        if let Some(text) = error_msg {
            err_resp.body.push_param(text).unwrap();
        }
        err_resp
    }
    /// Make a correctly addressed response with the correct response serial
    pub fn make_response(&self) -> crate::message_builder::MarshalledMessage {
        crate::message_builder::MarshalledMessage {
            typ: MessageType::Reply,
            dynheader: DynamicHeader {
                interface: None,
                member: None,
                object: None,
                destination: self.sender.clone(),
                serial: None,
                num_fds: None,
                sender: None,
                signature: None,
                response_serial: self.serial,
                error_name: None,
            },
            raw_fds: Vec::new(),
            flags: 0,
            body: crate::message_builder::MarshalledMessageBody::new(),
        }
    }
}

/// A message with all the different fields it may or may not have
#[derive(Debug, Clone)]
pub struct Message<'a, 'e> {
    pub typ: MessageType,
    pub flags: u8,

    // dynamic header
    pub dynheader: DynamicHeader,

    // body
    pub params: Vec<Param<'a, 'e>>,

    // out of band data
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
            dynheader: DynamicHeader::default(),
            flags: 0,
            raw_fds: Vec::new(),
            typ: MessageType::Invalid,
            params: Vec::new(),
        }
    }

    pub fn set_interface(&mut self, interface: String) {
        self.dynheader.interface = Some(interface);
    }
    pub fn set_member(&mut self, member: String) {
        self.dynheader.member = Some(member);
    }
    pub fn set_object(&mut self, object: String) {
        self.dynheader.object = Some(object);
    }
    pub fn set_destination(&mut self, dest: String) {
        self.dynheader.destination = Some(dest);
    }
    pub fn push_params<P: Into<Param<'a, 'e>>>(&mut self, params: Vec<P>) {
        self.params
            .extend(params.into_iter().map(std::convert::Into::into));
    }
    pub fn push_param<P: Into<Param<'a, 'e>>>(&mut self, param: P) {
        self.params.push(param.into());
    }

    /// Make a correctly addressed response with the correct response serial
    /// This is identical to calling [`self.dynheader.make_response()`].
    ///
    /// [`self.dynheader.make_response()`]: ./struct.DynamicHeader.html#method.make_response
    #[inline]
    pub fn make_response(&self) -> crate::message_builder::MarshalledMessage {
        self.dynheader.make_response()
    }

    pub fn set_flag(&mut self, flag: HeaderFlags) {
        flag.set(&mut self.flags)
    }
    pub fn unset_flag(&mut self, flag: HeaderFlags) {
        flag.unset(&mut self.flags)
    }
    pub fn toggle_flag(&mut self, flag: HeaderFlags) {
        flag.toggle(&mut self.flags)
    }

    pub fn sig(&self) -> Vec<signature::Type> {
        self.params.iter().map(|p| p.sig()).collect()
    }

    pub fn add_param<P: Into<Param<'a, 'e>>>(&mut self, p: P) {
        self.params.push(p.into());
    }
    pub fn add_param2<P1: Into<Param<'a, 'e>>, P2: Into<Param<'a, 'e>>>(&mut self, p1: P1, p2: P2) {
        self.params.push(p1.into());
        self.params.push(p2.into());
    }
    pub fn add_param3<P1: Into<Param<'a, 'e>>, P2: Into<Param<'a, 'e>>, P3: Into<Param<'a, 'e>>>(
        &mut self,
        p1: P1,
        p2: P2,
        p3: P3,
    ) {
        self.params.push(p1.into());
        self.params.push(p2.into());
        self.params.push(p3.into());
    }
}

#[derive(Copy, Clone)]
pub enum HeaderFlags {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

impl HeaderFlags {
    pub fn into_raw(self) -> u8 {
        match self {
            HeaderFlags::NoReplyExpected => 1,
            HeaderFlags::NoAutoStart => 2,
            HeaderFlags::AllowInteractiveAuthorization => 4,
        }
    }

    pub fn is_set(self, flags: u8) -> bool {
        flags & self.into_raw() == 1
    }

    pub fn set(self, flags: &mut u8) {
        *flags |= self.into_raw()
    }

    pub fn unset(self, flags: &mut u8) {
        *flags &= 0xFF - self.into_raw()
    }
    pub fn toggle(self, flags: &mut u8) {
        if self.is_set(*flags) {
            self.unset(flags)
        } else {
            self.set(flags)
        }
    }
}

pub type Result<T> = std::result::Result<T, crate::Error>;
