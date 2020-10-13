//! Messages that have been completetly unmarshalled

use crate::message_builder::{DynamicHeader, HeaderFlags, MessageType};
use crate::params::*;
use crate::signature;
use std::os::unix::io::RawFd;

/// A message with all the different fields it may or may not have
/// and only Params as the body
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

pub type Result<T> = std::result::Result<T, crate::Error>;
