//! Helps in building messages conveniently

use crate::message;
use crate::params;
use crate::params::Param;

#[derive(Default)]
pub struct MessageBuilder<'a, 'e> {
    msg: message::Message<'a, 'e>,
}

pub struct CallBuilder<'a, 'e> {
    msg: message::Message<'a, 'e>,
}
pub struct SignalBuilder<'a, 'e> {
    msg: message::Message<'a, 'e>,
}

impl<'a, 'e> MessageBuilder<'a, 'e> {
    pub fn new() -> MessageBuilder<'a, 'e> {
        MessageBuilder {
            msg: message::Message::new(),
        }
    }

    pub fn call(mut self, member: String) -> CallBuilder<'a, 'e> {
        self.msg.typ = message::MessageType::Call;
        self.msg.member = Some(member);
        CallBuilder { msg: self.msg }
    }
    pub fn signal(
        mut self,
        interface: String,
        member: String,
        object: String,
    ) -> SignalBuilder<'a, 'e> {
        self.msg.typ = message::MessageType::Signal;
        self.msg.member = Some(member);
        self.msg.interface = Some(interface);
        self.msg.object = Some(object);
        SignalBuilder { msg: self.msg }
    }
}

impl<'a, 'e> CallBuilder<'a, 'e> {
    pub fn on(mut self, object_path: String) -> Self {
        self.msg.object = Some(object_path);
        self
    }

    pub fn with_interface(mut self, interface: String) -> Self {
        self.msg.interface = Some(interface);
        self
    }

    pub fn at(mut self, destination: String) -> Self {
        self.msg.destination = Some(destination);
        self
    }

    pub fn with_params<P: Into<params::Param<'a, 'e>>>(mut self, params: Vec<P>) -> Self {
        self.msg.push_params(params);
        self
    }

    pub fn build(self) -> message::Message<'a, 'e> {
        self.msg
    }

    pub fn add_param<P: Into<Param<'a, 'e>>>(&mut self, p: P) {
        self.msg.add_param(p);
    }
    pub fn add_param2<P1: Into<Param<'a, 'e>>, P2: Into<Param<'a, 'e>>>(&mut self, p1: P1, p2: P2) {
        self.msg.add_param2(p1, p2);
    }
    pub fn add_param3<P1: Into<Param<'a, 'e>>, P2: Into<Param<'a, 'e>>, P3: Into<Param<'a, 'e>>>(
        &mut self,
        p1: P1,
        p2: P2,
        p3: P3,
    ) {
        self.msg.add_param3(p1, p2, p3);
    }
}

impl<'a, 'e> SignalBuilder<'a, 'e> {
    pub fn to(mut self, destination: String) -> Self {
        self.msg.destination = Some(destination);
        self
    }

    pub fn with_params(mut self, params: Vec<params::Param<'a, 'e>>) -> Self {
        self.msg.params.extend(params);
        self
    }

    pub fn build(self) -> message::Message<'a, 'e> {
        self.msg
    }
}
