//! Helps in building messages conveniently

use crate::message;
use crate::wire::marshal_trait::Marshal;
use std::os::unix::io::RawFd;

#[derive(Default)]
pub struct MessageBuilder {
    msg: OutMessage,
}

pub struct CallBuilder {
    msg: OutMessage,
}
pub struct SignalBuilder {
    msg: OutMessage,
}

impl MessageBuilder {
    pub fn new() -> MessageBuilder {
        MessageBuilder {
            msg: OutMessage::new(),
        }
    }

    pub fn call(mut self, member: String) -> CallBuilder {
        self.msg.typ = message::MessageType::Call;
        self.msg.member = Some(member);
        CallBuilder { msg: self.msg }
    }
    pub fn signal(mut self, interface: String, member: String, object: String) -> SignalBuilder {
        self.msg.typ = message::MessageType::Signal;
        self.msg.member = Some(member);
        self.msg.interface = Some(interface);
        self.msg.object = Some(object);
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

    pub fn build(self) -> OutMessage {
        self.msg
    }
}

impl SignalBuilder {
    pub fn to(mut self, destination: String) -> Self {
        self.msg.destination = Some(destination);
        self
    }

    pub fn build(self) -> OutMessage {
        self.msg
    }
}

#[derive(Debug)]
pub struct OutMessage {
    pub body: OutMessageBody,

    // dynamic header
    pub interface: Option<String>,
    pub member: Option<String>,
    pub object: Option<String>,
    pub destination: Option<String>,
    pub serial: Option<u32>,
    pub sender: Option<String>,
    pub error_name: Option<String>,
    pub response_serial: Option<u32>,
    pub num_fds: Option<u32>,

    // out of band data
    pub raw_fds: Vec<RawFd>,

    pub typ: message::MessageType,
    pub flags: u8,
}

impl Default for OutMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl OutMessage {
    pub fn get_buf(&self) -> &[u8] {
        &self.body.buf
    }
    pub fn get_sig(&self) -> &str {
        &self.body.sig
    }
    pub fn new() -> Self {
        OutMessage {
            typ: message::MessageType::Invalid,
            interface: None,
            member: None,
            object: None,
            destination: None,
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            response_serial: None,
            sender: None,
            error_name: None,
            flags: 0,

            body: OutMessageBody::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct OutMessageBody {
    buf: Vec<u8>,
    sig: String,
}

pub fn marshal_as_variant<P: Marshal>(
    p: P,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> Result<(), message::Error> {
    let mut sig_str = String::new();
    p.signature().to_str(&mut sig_str);
    crate::wire::util::pad_to_align(p.alignment(), buf);
    crate::wire::marshal_base::marshal_base_param(
        message::ByteOrder::LittleEndian,
        &crate::params::Base::Signature(sig_str),
        buf,
    )
    .unwrap();
    p.marshal(byteorder, buf)?;
    Ok(())
}

impl OutMessageBody {
    pub fn new() -> Self {
        OutMessageBody {
            buf: Vec::new(),
            sig: String::new(),
        }
    }

    pub fn push_param<P: Marshal>(&mut self, p: P) -> Result<(), message::Error> {
        p.marshal(message::ByteOrder::LittleEndian, &mut self.buf)?;
        p.signature().to_str(&mut self.sig);
        Ok(())
    }

    pub fn push_param2<P1: Marshal, P2: Marshal>(
        &mut self,
        p1: P1,
        p2: P2,
    ) -> Result<(), message::Error> {
        self.push_param(p1)?;
        self.push_param(p2)?;
        Ok(())
    }

    pub fn push_param3<P1: Marshal, P2: Marshal, P3: Marshal>(
        &mut self,
        p1: P1,
        p2: P2,
        p3: P3,
    ) -> Result<(), message::Error> {
        self.push_param(p1)?;
        self.push_param(p2)?;
        self.push_param(p3)?;
        Ok(())
    }

    pub fn push_param4<P1: Marshal, P2: Marshal, P3: Marshal, P4: Marshal>(
        &mut self,
        p1: P1,
        p2: P2,
        p3: P3,
        p4: P4,
    ) -> Result<(), message::Error> {
        self.push_param(p1)?;
        self.push_param(p2)?;
        self.push_param(p3)?;
        self.push_param(p4)?;
        Ok(())
    }

    pub fn push_param5<P1: Marshal, P2: Marshal, P3: Marshal, P4: Marshal, P5: Marshal>(
        &mut self,
        p1: P1,
        p2: P2,
        p3: P3,
        p4: P4,
        p5: P5,
    ) -> Result<(), message::Error> {
        self.push_param(p1)?;
        self.push_param(p2)?;
        self.push_param(p3)?;
        self.push_param(p4)?;
        self.push_param(p5)?;
        Ok(())
    }

    pub fn push_params<P: Marshal>(&mut self, params: &[P]) -> Result<(), message::Error> {
        for p in params {
            self.push_param(p)?;
        }
        Ok(())
    }

    pub fn push_variant<P: Marshal>(&mut self, p: P) -> Result<(), message::Error> {
        self.sig.push('v');
        marshal_as_variant(p, message::ByteOrder::LittleEndian, &mut self.buf)
    }

    pub fn push_empty_array(&mut self, elem_sig: crate::signature::Type) {
        self.sig.push('a');
        elem_sig.to_str(&mut self.sig);

        // always align to 4
        crate::wire::util::pad_to_align(4, &mut self.buf);
        self.buf.push(0);
        self.buf.push(0);
        self.buf.push(0);
        self.buf.push(0);
    }

    pub fn push_empty_dict(
        &mut self,
        key_sig: crate::signature::Base,
        val_sig: crate::signature::Type,
    ) {
        self.sig.push('a');
        self.sig.push('{');
        key_sig.to_str(&mut self.sig);
        val_sig.to_str(&mut self.sig);
        self.sig.push('}');

        // always align to 4
        crate::wire::util::pad_to_align(4, &mut self.buf);
        self.buf.push(0);
        self.buf.push(0);
        self.buf.push(0);
        self.buf.push(0);
    }
}

#[test]
fn test_marshal_trait() {
    let mut body = OutMessageBody::new();
    let bytes: &[&[_]] = &[&[4u64]];
    body.push_param(bytes).unwrap();

    assert_eq!(
        vec![12, 0, 0, 0, 8, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0],
        body.buf
    );
    assert_eq!(body.sig.as_str(), "aat");

    let mut body = OutMessageBody::new();
    let mut map = std::collections::HashMap::new();
    map.insert("a", 4u32);

    body.push_param(&map).unwrap();
    assert_eq!(
        vec![12, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, b'a', 0, 0, 0, 4, 0, 0, 0,],
        body.buf
    );
    assert_eq!(body.sig.as_str(), "a{su}");

    let mut body = OutMessageBody::new();
    body.push_param((11u64, "str", true)).unwrap();
    assert_eq!(body.sig.as_str(), "(tsb)");
    assert_eq!(
        vec![11, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, b's', b't', b'r', 0, 1, 0, 0, 0,],
        body.buf
    );

    struct MyStruct {
        x: u64,
        y: String,
    }

    impl Marshal for &MyStruct {
        fn marshal(
            &self,
            byteorder: message::ByteOrder,
            buf: &mut Vec<u8>,
        ) -> Result<(), message::Error> {
            // always align to 8
            crate::wire::util::pad_to_align(8, buf);
            self.x.marshal(byteorder, buf)?;
            self.y.marshal(byteorder, buf)?;
            Ok(())
        }
        fn signature(&self) -> crate::signature::Type {
            crate::signature::Type::Container(crate::signature::Container::Struct(vec![
                self.x.signature(),
                self.y.signature(),
            ]))
        }

        fn alignment(&self) -> usize {
            8
        }
    }

    let mut body = OutMessageBody::new();
    body.push_param(&MyStruct {
        x: 100,
        y: "A".to_owned(),
    })
    .unwrap();
    assert_eq!(body.sig.as_str(), "(ts)");
    assert_eq!(
        vec![100, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, b'A', 0,],
        body.buf
    );

    let mut body = OutMessageBody::new();
    let mut map = std::collections::HashMap::new();
    let mut map2 = std::collections::HashMap::new();
    map.insert("a", 4u32);
    map2.insert("a", &map);

    body.push_param(&map2).unwrap();
    body.push_empty_dict(
        crate::signature::Base::String,
        crate::signature::Type::Base(crate::signature::Base::Uint32),
    );
    assert_eq!(body.sig.as_str(), "a{sa{su}}a{su}");
    assert_eq!(
        vec![
            28, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, b'a', 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, b'a', 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0
        ],
        body.buf
    );
}
