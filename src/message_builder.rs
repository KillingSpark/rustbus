use crate::message;

pub struct MessageBuilder {
    msg: message::Message,
}

pub struct CallBuilder {
    msg: message::Message,
}
pub struct SignalBuilder {
    msg: message::Message,
}

impl MessageBuilder {
    pub fn new() -> MessageBuilder {
        MessageBuilder {
            msg: message::Message::new(),
        }
    }

    pub fn call(mut self, member: String) -> CallBuilder {
        self.msg.typ = message::MessageType::Call;
        self.msg.member = Some(member);
        CallBuilder { msg: self.msg }
    }
    pub fn signal(mut self, interface: String, member: String) -> SignalBuilder {
        self.msg.typ = message::MessageType::Signal;
        self.msg.member = Some(member);
        self.msg.interface = Some(interface);
        SignalBuilder { msg: self.msg }
    }
}

impl CallBuilder {
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

    pub fn with_params(mut self, params: Vec<message::Param>) -> Self {
        self.msg.params.extend(params);
        self
    }

    pub fn build(self) -> message::Message {
        self.msg
    }
}

impl SignalBuilder {
    pub fn with_interface(mut self, interface: String) -> Self {
        self.msg.interface = Some(interface);
        self
    }

    pub fn at(mut self, destination: String) -> Self {
        self.msg.destination = Some(destination);
        self
    }

    pub fn with_params(mut self, params: Vec<message::Param>) -> Self {
        self.msg.params.extend(params);
        self
    }

    pub fn build(self) -> message::Message {
        self.msg
    }
}
