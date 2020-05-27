//! Helps in building messages conveniently

use crate::message;
use crate::params;
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
        p.marshal(message::ByteOrder::LittleEndian, &mut self.buf)?;
        p.signature().to_str(&mut self.sig);
        Ok(())
    }

    pub fn push_empty_array(&mut self, elem_sig: crate::signature::Type) {
        self.sig.push('a');
        elem_sig.to_str(&mut self.sig);

        // always align to 4
        let pad_size = self.buf.len() % 4;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            self.buf.push(0);
        }
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
        let pad_size = self.buf.len() % 4;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            self.buf.push(0);
        }
        self.buf.push(0);
        self.buf.push(0);
        self.buf.push(0);
        self.buf.push(0);
    }
}

impl Marshal for () {
    fn marshal(
        &self,
        _byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        let pad_size = buf.len() % 8;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }
        Ok(())
    }
    fn signature(&self) -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![]))
    }

    fn alignment(&self) -> usize {
        8
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
        if self.is_empty() {
            return Err(message::Error::EmptyArray);
        }

        // always align to 4
        let pad_size = buf.len() % 4;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }

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
        if self.is_empty() {
            return Err(message::Error::EmptyDict);
        }

        // always align to 4
        let pad_size = buf.len() % 4;
        eprintln!("pad_size: {}", pad_size);
        for _ in 0..pad_size {
            buf.push(0);
        }

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

impl<'a> Marshal for &params::Param<'a, 'a> {
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

impl Marshal for String {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        let b: params::Base = self.as_str().into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature(&self) -> crate::signature::Type {
        let b: params::Base = self.as_str().into();
        b.sig()
    }
    fn alignment(&self) -> usize {
        let b: params::Base = self.as_str().into();
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
