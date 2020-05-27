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

pub struct OutMessageBody {
    buf: Vec<u8>,
    sig: String,
}

impl OutMessageBody {
    pub fn new() -> Self {
        OutMessageBody {
            buf: Vec::new(),
            sig: String::new(),
        }
    }

    pub fn push_param<P: Marshal>(&mut self, p: P) -> Result<(), message::Error> {
        p.signature().to_str(&mut self.sig);
        p.marshal(message::ByteOrder::LittleEndian, &mut self.buf)
    }
}

impl<E: Marshal> Marshal for (E,) {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        let pad_size = buf.len() % 8;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }
        self.0.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature(&self) -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![self
            .0
            .signature()]))
    }

    fn alignment(&self) -> usize {
        8
    }
}

impl<E1: Marshal, E2: Marshal> Marshal for (E1, E2) {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        let pad_size = buf.len() % 8;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature(&self) -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            self.0.signature(),
            self.1.signature(),
        ]))
    }

    fn alignment(&self) -> usize {
        8
    }
}

impl<E1: Marshal, E2: Marshal, E3: Marshal> Marshal for (E1, E2, E3) {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        let pad_size = buf.len() % 8;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        self.2.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature(&self) -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            self.0.signature(),
            self.1.signature(),
            self.2.signature(),
        ]))
    }

    fn alignment(&self) -> usize {
        8
    }
}

impl<E1: Marshal, E2: Marshal, E3: Marshal, E4: Marshal> Marshal for (E1, E2, E3, E4) {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        let pad_size = buf.len() % 8;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        self.2.marshal(byteorder, buf)?;
        self.3.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature(&self) -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            self.0.signature(),
            self.1.signature(),
            self.2.signature(),
            self.3.signature(),
        ]))
    }

    fn alignment(&self) -> usize {
        8
    }
}

impl<E: Marshal> Marshal for &[E] {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let size_pos = buf.len();
        buf.push(0);
        buf.push(0);
        buf.push(0);
        buf.push(0);

        if self.len() > 0 && self[0].alignment() > 4 {
            let pad_size = buf.len() % self[0].alignment();
            eprintln!("pad_size: {}", pad_size);
            for _ in 0..pad_size {
                buf.push(0);
            }
        }

        let size_before = buf.len();
        for p in self.iter() {
            p.marshal(byteorder, buf)?;
        }
        let size_of_content = buf.len() - size_before;
        crate::wire::util::insert_u32(
            byteorder,
            size_of_content as u32,
            &mut buf[size_pos..size_pos + 4],
        );

        Ok(())
    }
    fn signature(&self) -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            self[0].signature(),
        )))
    }

    fn alignment(&self) -> usize {
        4
    }
}

impl<K: Marshal, V: Marshal> Marshal for &std::collections::HashMap<K, V> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let size_pos = buf.len();
        buf.push(0);
        buf.push(0);
        buf.push(0);
        buf.push(0);

        if self.len() > 0 {
            // always align to 8
            let pad_size = buf.len() % 8;
            eprintln!("pad_size: {}", pad_size);
            for _ in 0..pad_size {
                buf.push(0);
            }
        }

        let size_before = buf.len();
        for p in self.iter() {
            // always align to 8
            let pad_size = buf.len() % 8;
            eprintln!("pad_size: {}", pad_size);
            for _ in 0..pad_size {
                buf.push(0);
            }
            p.0.marshal(byteorder, buf)?;
            p.1.marshal(byteorder, buf)?;
        }
        let size_of_content = buf.len() - size_before;
        crate::wire::util::insert_u32(
            byteorder,
            size_of_content as u32,
            &mut buf[size_pos..size_pos + 4],
        );

        Ok(())
    }

    fn signature(&self) -> crate::signature::Type {
        let ks = self.keys().nth(0).unwrap().signature();
        let vs = self.values().nth(0).unwrap().signature();
        if let crate::signature::Type::Base(ks) = ks {
            crate::signature::Type::Container(crate::signature::Container::Dict(ks, Box::new(vs)))
        } else {
            panic!("Ivalid key sig")
        }
    }

    fn alignment(&self) -> usize {
        4
    }
}

pub trait Marshal {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error>;

    fn signature(&self) -> crate::signature::Type;
    fn alignment(&self) -> usize;
}

impl<'a> Marshal for params::Param<'a, 'a> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::marshal_container::marshal_param(self, byteorder, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        self.sig()
    }
    fn alignment(&self) -> usize {
        self.sig().get_alignment()
    }
}

impl Marshal for u64 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.into();
        b.sig().get_alignment()
    }
}

impl Marshal for u32 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.into();
        b.sig().get_alignment()
    }
}

impl Marshal for u16 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.into();
        b.sig().get_alignment()
    }
}

impl Marshal for u8 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.into();
        b.sig().get_alignment()
    }
}

impl Marshal for bool {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.into();
        b.sig().get_alignment()
    }
}

impl Marshal for dyn AsRef<str> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.as_ref().into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.as_ref().into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.as_ref().into();
        b.sig().get_alignment()
    }
}

impl Marshal for &str {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = (*self).into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = (*self).into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = (*self).into();
        b.sig().get_alignment()
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
            let pad_size = buf.len() % 8;
            eprintln!("pad_size: {}", pad_size);
            for _ in 0..pad_size {
                buf.push(0);
            }
            self.x.marshal(byteorder, buf)?;
            self.y.as_str().marshal(byteorder, buf)?;
            Ok(())
        }
        fn signature(&self) -> crate::signature::Type {
            crate::signature::Type::Container(crate::signature::Container::Struct(vec![
                self.x.signature(),
                self.y.as_str().signature(),
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
}
