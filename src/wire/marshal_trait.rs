//! Marshal trait and implementations for the basic types

use crate::message;
use crate::params;

/// The Marshal trait allows to push any type onto an message_builder::OutMessage as a parameter.
/// There are some useful implementations here for slices and hashmaps which map to arrays and dicts in the dbus message.
///
/// The way dbus structs are represented is with rust tuples. This lib provides Marshal impls for tuples with up to 5 elements.
/// If you need more you can just copy the impl and extend it to how many different entries you need.
///
/// Also you can implement Marshal for your own structs:
/// ```rust
/// struct MyStruct {
///     x: u64,
///     y: String,
/// }
///
/// use rustbus::message;
/// use rustbus::signature;
/// use rustbus::wire::util;
/// use rustbus::Marshal;
/// impl Marshal for &MyStruct {
///     fn marshal(
///         &self,
///         byteorder: message::ByteOrder,
///         buf: &mut Vec<u8>,
///     ) -> Result<(), message::Error> {
///         // always align to 8
///         util::pad_to_align(8, buf);
///         self.x.marshal(byteorder, buf)?;
///         self.y.marshal(byteorder, buf)?;
///         Ok(())
///     }
///     fn signature() -> signature::Type {
///         signature::Type::Container(signature::Container::Struct(vec![
///             u64::signature(),
///             String::signature(),
///         ]))
///     }
///
///     fn alignment() -> usize {
///         8
///     }
/// }
/// ```
/// # Implementing for your own structs
/// There are some rules you need to follow, or the messages will be malformed:
/// 1. Structs need to be aligned to 8 bytes. Use `wire::util::pad_to_align(8, buf)` to do that. If your type is marshalled as a primitive type
///     you still need to align to that types alignment.
/// 1. If you write your own dict type, you need to align every key-value pair at 8 bytes like a struct
/// 1. The signature needs to be correct, or the message will be malformed
/// 1. The alignment must report the correct number. This does not need to be a constant like in the example, but it needs to be consistent with the type
///     the signature() function returns. If you are not sure, just use Self::signature().get_alignment().
pub trait Marshal {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error>;

    fn signature() -> crate::signature::Type;
    fn alignment() -> usize;
}

impl<P: Marshal> Marshal for &P {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        (*self).marshal(byteorder, buf)
    }

    fn signature() -> crate::signature::Type {
        P::signature()
    }
    fn alignment() -> usize {
        P::alignment()
    }
}

impl Marshal for () {
    fn marshal(
        &self,
        _byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        crate::wire::util::pad_to_align(8, buf);
        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![]))
    }

    fn alignment() -> usize {
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
        crate::wire::util::pad_to_align(8, buf);
        self.0.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![E::signature()]))
    }

    fn alignment() -> usize {
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
        crate::wire::util::pad_to_align(8, buf);
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            E1::signature(),
            E2::signature(),
        ]))
    }

    fn alignment() -> usize {
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
        crate::wire::util::pad_to_align(8, buf);
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        self.2.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            E1::signature(),
            E2::signature(),
            E3::signature(),
        ]))
    }

    fn alignment() -> usize {
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
        crate::wire::util::pad_to_align(8, buf);
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        self.2.marshal(byteorder, buf)?;
        self.3.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            E1::signature(),
            E2::signature(),
            E3::signature(),
            E4::signature(),
        ]))
    }

    fn alignment() -> usize {
        8
    }
}

impl<E1: Marshal, E2: Marshal, E3: Marshal, E4: Marshal, E5: Marshal> Marshal
    for (E1, E2, E3, E4, E5)
{
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 8
        crate::wire::util::pad_to_align(8, buf);
        self.0.marshal(byteorder, buf)?;
        self.1.marshal(byteorder, buf)?;
        self.2.marshal(byteorder, buf)?;
        self.3.marshal(byteorder, buf)?;
        self.4.marshal(byteorder, buf)?;
        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Struct(vec![
            E1::signature(),
            E2::signature(),
            E3::signature(),
            E4::signature(),
            E5::signature(),
        ]))
    }

    fn alignment() -> usize {
        8
    }
}

impl<E: Marshal> Marshal for &[E] {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 4
        crate::wire::util::pad_to_align(4, buf);

        let size_pos = buf.len();
        buf.push(0);
        buf.push(0);
        buf.push(0);
        buf.push(0);

        if self.is_empty() {
            return Ok(());
        }
        crate::wire::util::pad_to_align(E::alignment(), buf);

        // we can reserve at least one byte per entry without wasting memory, and save at least a few reallocations of buf
        buf.reserve(self.len());
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
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            E::signature(),
        )))
    }

    fn alignment() -> usize {
        4
    }
}

/// **_!!! This assumes that you are marshalling to the platforms byteorder !!!_**
///
/// It just memcpy's the content of the array into the message. This is fine for all integer types, but you cannot use it for structs,
/// even if they are copy!
/// They might have padding to be correctly aligned in the slice. I would recommend to only use this for marshalling
/// big integer arrays but I cannot express this in the type system cleanly so here is a comment.
pub struct OptimizedMarshal<'a, E: Copy + Marshal>(pub &'a [E]);
impl<'a, E: Copy + Marshal> Marshal for OptimizedMarshal<'a, E> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 4
        crate::wire::util::pad_to_align(4, buf);

        let size_pos = buf.len();
        buf.push(0);
        buf.push(0);
        buf.push(0);
        buf.push(0);

        if self.0.is_empty() {
            return Ok(());
        }
        crate::wire::util::pad_to_align(E::alignment(), buf);

        let size_of_content = std::mem::size_of::<E>() * self.0.len();
        let len_before_resize = buf.len();
        buf.resize(buf.len() + size_of_content, 0);
        unsafe {
            let content_ptr = buf.as_mut_ptr().add(len_before_resize);
            std::ptr::copy_nonoverlapping(
                self.0.as_ptr() as *const u8,
                content_ptr,
                size_of_content,
            );
        }
        crate::wire::util::insert_u32(
            byteorder,
            size_of_content as u32,
            &mut buf[size_pos..size_pos + 4],
        );

        Ok(())
    }
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            E::signature(),
        )))
    }

    fn alignment() -> usize {
        4
    }
}

#[test]
fn verify_optimized_arrays() {
    // marshal array of u64 optimized and non-optimized and compare
    let mut buf_normal = Vec::new();
    let mut buf_optimized = Vec::new();
    let arru64: Vec<u64> = vec![1, 2, 3, 4, 5, 6, u64::MAX, u64::MAX / 2, u64::MAX / 1024];
    arru64
        .as_slice()
        .marshal(message::ByteOrder::LittleEndian, &mut buf_normal)
        .unwrap();
    OptimizedMarshal(arru64.as_slice())
        .marshal(message::ByteOrder::LittleEndian, &mut buf_optimized)
        .unwrap();
    assert_eq!(buf_normal, buf_optimized);

    // marshal array of u8 optimized and non-optimized and compare
    let mut buf_normal = Vec::new();
    let mut buf_otpimized = Vec::new();
    let arru8: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 255, 128, 80, 180];
    arru8
        .as_slice()
        .marshal(message::ByteOrder::LittleEndian, &mut buf_normal)
        .unwrap();
    OptimizedMarshal(arru8.as_slice())
        .marshal(message::ByteOrder::LittleEndian, &mut buf_otpimized)
        .unwrap();
    assert_eq!(buf_normal, buf_otpimized);

    // check that empty arrays work as expected
    let empty: Vec<u8> = Vec::new();
    let x = crate::params::Container::make_array("y", empty.clone().into_iter()).unwrap();
    let mut buf_old = Vec::new();
    let mut buf_normal = Vec::new();
    let mut buf_otpimized = Vec::new();
    empty
        .as_slice()
        .marshal(message::ByteOrder::LittleEndian, &mut buf_normal)
        .unwrap();
    OptimizedMarshal(empty.as_slice())
        .marshal(message::ByteOrder::LittleEndian, &mut buf_otpimized)
        .unwrap();
    crate::wire::marshal_container::marshal_container_param(
        &x,
        message::ByteOrder::LittleEndian,
        &mut buf_old,
    )
    .unwrap();
    assert_eq!(buf_normal, buf_otpimized);
    assert_eq!(buf_normal, buf_old);
    assert_eq!(vec![0, 0, 0, 0], buf_old);
}

impl<K: Marshal, V: Marshal> Marshal for &std::collections::HashMap<K, V> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        // always align to 4
        crate::wire::util::pad_to_align(4, buf);

        let size_pos = buf.len();
        buf.push(0);
        buf.push(0);
        buf.push(0);
        buf.push(0);

        if self.is_empty() {
            return Ok(());
        }
        // always align to 8
        crate::wire::util::pad_to_align(8, buf);

        let size_before = buf.len();
        for p in self.iter() {
            // always align to 8
            crate::wire::util::pad_to_align(8, buf);
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

    fn signature() -> crate::signature::Type {
        let ks = K::signature();
        let vs = V::signature();
        if let crate::signature::Type::Base(ks) = ks {
            crate::signature::Type::Container(crate::signature::Container::Dict(ks, Box::new(vs)))
        } else {
            panic!("Ivalid key sig")
        }
    }

    fn alignment() -> usize {
        4
    }
}

impl Marshal for u64 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint64)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
impl Marshal for i64 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int64)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

impl Marshal for u32 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint32)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
impl Marshal for i32 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int32)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

impl Marshal for u16 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Uint16)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
impl Marshal for i16 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Int16)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

impl Marshal for u8 {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Byte)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

impl Marshal for bool {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        let b: params::Base = self.into();
        crate::wire::marshal_base::marshal_base_param(byteorder, &b, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Boolean)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

impl Marshal for String {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        crate::wire::util::write_string(self.as_str(), byteorder, buf);
        Ok(())
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::String)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

impl Marshal for &str {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        crate::wire::util::pad_to_align(Self::alignment(), buf);
        crate::wire::util::write_string(self, byteorder, buf);
        Ok(())
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::String)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

pub struct ObjectPath<'a>(&'a str);
impl<'a> ObjectPath<'a> {
    pub fn new(path: &'a str) -> Result<Self, message::Error> {
        crate::params::validate_object_path(path)?;
        Ok(ObjectPath(path))
    }
}
impl Marshal for ObjectPath<'_> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        self.0.marshal(byteorder, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::ObjectPath)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

pub struct Signature<'a>(&'a str);
impl<'a> Signature<'a> {
    pub fn new(sig: &'a str) -> Result<Self, message::Error> {
        crate::params::validate_signature(sig)?;
        Ok(Signature(sig))
    }
}
impl Marshal for Signature<'_> {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        self.0.marshal(byteorder, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::Signature)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}
pub struct UnixFd(pub u32);
impl Marshal for UnixFd {
    fn marshal(
        &self,
        byteorder: message::ByteOrder,
        buf: &mut Vec<u8>,
    ) -> Result<(), message::Error> {
        self.0.marshal(byteorder, buf)
    }

    fn signature() -> crate::signature::Type {
        crate::signature::Type::Base(crate::signature::Base::UnixFd)
    }
    fn alignment() -> usize {
        Self::signature().get_alignment()
    }
}

#[test]
fn test_trait_signature_creation() {
    let mut msg = crate::message_builder::OutMessage::new();
    let body = &mut msg.body;

    body.push_param("a").unwrap();
    body.push_param(ObjectPath::new("/a/b").unwrap()).unwrap();
    body.push_param(Signature::new("(a{su})").unwrap()).unwrap();
    body.push_param(UnixFd(10)).unwrap();
    body.push_param(true).unwrap();
    body.push_param(0u8).unwrap();
    body.push_param(0u16).unwrap();
    body.push_param(0u32).unwrap();
    body.push_param(0u64).unwrap();
    body.push_param(0i16).unwrap();
    body.push_param(0i32).unwrap();
    body.push_param(0i64).unwrap();
    body.push_param(&[0u8][..]).unwrap();

    let map: std::collections::HashMap<String, (u64, u32, u16, u8)> =
        std::collections::HashMap::new();
    body.push_param(&map).unwrap();

    assert_eq!("soghbyqutnixaya{s(tuqy)}", msg.get_sig());
}
