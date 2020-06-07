//! An unfinished first try at implementing an unmarshal trait. To apply this to message decoding signature checking would have to be done in some way...

use crate::message::ByteOrder;
use crate::wire::marshal_trait::Signature;
use crate::wire::unmarshal;
use crate::wire::util;

pub trait Unmarshal<'r, 'buf: 'r>: Sized + Signature {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self>;
}

pub fn unmarshal<'r, 'buf: 'r, T: Unmarshal<'r, 'buf>>(
    byteorder: ByteOrder,
    buf: &'buf [u8],
    offset: usize,
) -> unmarshal::UnmarshalResult<T> {
    T::unmarshal(byteorder, buf, offset)
}

#[test]
fn test_generic_unmarshal() {
    use crate::Marshal;

    // annotate the receiver with a type &str to unmarshal a &str
    let mut buf = Vec::new();
    "ABCD".marshal(ByteOrder::LittleEndian, &mut buf).unwrap();
    let _s: &str = unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap().1;

    // annotate the receiver with a type bool to unmarshal a bool
    buf.clear();
    true.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();
    let _b: bool = unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap().1;

    // can also use turbofish syntax
    buf.clear();
    0i32.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();
    let _i = unmarshal::<i32>(ByteOrder::LittleEndian, &buf, 0)
        .unwrap()
        .1;

    // No type info on let arg = unmarshal(...) is needed if it can be derived by other means
    buf.clear();
    fn x(_arg: (i32, i32, &str)) {};
    (0, 0, "ABCD")
        .marshal(ByteOrder::LittleEndian, &mut buf)
        .unwrap();
    let arg = unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap().1;
    x(arg);
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for () {
    fn unmarshal(
        _byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        Ok((padding, ()))
    }
}

impl<'r, 'buf: 'r, E1> Unmarshal<'r, 'buf> for (E1,)
where
    E1: Unmarshal<'r, 'buf> + Sized,
{
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val1) = E1::unmarshal(byteorder, buf, offset)?;
        Ok((bytes + padding, (val1,)))
    }
}

impl<'r, 'buf: 'r, E1, E2> Unmarshal<'r, 'buf> for (E1, E2)
where
    E1: Unmarshal<'r, 'buf> + Sized,
    E2: Unmarshal<'r, 'buf> + Sized,
{
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let mut total_bytes = 0;

        let padding = util::align_offset(Self::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val1) = E1::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        let padding = util::align_offset(E2::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val2) = E2::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        Ok((total_bytes, (val1, val2)))
    }
}

impl<'r, 'buf: 'r, E1, E2, E3> Unmarshal<'r, 'buf> for (E1, E2, E3)
where
    E1: Unmarshal<'r, 'buf> + Sized,
    E2: Unmarshal<'r, 'buf> + Sized,
    E3: Unmarshal<'r, 'buf> + Sized,
{
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let mut total_bytes = 0;

        let padding = util::align_offset(Self::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val1) = E1::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        let padding = util::align_offset(E2::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val2) = E2::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        let padding = util::align_offset(E3::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val3) = E3::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        Ok((total_bytes, (val1, val2, val3)))
    }
}

impl<'r, 'buf: 'r, E1, E2, E3, E4> Unmarshal<'r, 'buf> for (E1, E2, E3, E4)
where
    E1: Unmarshal<'r, 'buf> + Sized,
    E2: Unmarshal<'r, 'buf> + Sized,
    E3: Unmarshal<'r, 'buf> + Sized,
    E4: Unmarshal<'r, 'buf> + Sized,
{
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let mut total_bytes = 0;

        let padding = util::align_offset(Self::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val1) = E1::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        let padding = util::align_offset(E2::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val2) = E2::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        let padding = util::align_offset(E3::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val3) = E3::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        let padding = util::align_offset(E4::alignment(), buf, offset + total_bytes)?;
        total_bytes += padding;
        let (bytes, val4) = E4::unmarshal(byteorder, buf, offset + total_bytes)?;
        total_bytes += bytes;

        Ok((total_bytes, (val1, val2, val3, val4)))
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for u64 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::parse_u64(&buf[offset..], byteorder)?;
        Ok((bytes + padding, val))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for u32 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::parse_u32(&buf[offset..], byteorder)?;
        Ok((bytes + padding, val))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for u16 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::parse_u16(&buf[offset..], byteorder)?;
        Ok((bytes + padding, val))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for i64 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) =
            util::parse_u64(&buf[offset..], byteorder).map(|(bytes, val)| (bytes, val as i64))?;
        Ok((bytes + padding, val))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for i32 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) =
            util::parse_u32(&buf[offset..], byteorder).map(|(bytes, val)| (bytes, val as i32))?;
        Ok((bytes + padding, val))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for i16 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) =
            util::parse_u16(&buf[offset..], byteorder).map(|(bytes, val)| (bytes, val as i16))?;
        Ok((bytes + padding, val))
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for u8 {
    fn unmarshal(
        _byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        if buf[offset..].is_empty() {
            return Err(crate::wire::unmarshal::Error::NotEnoughBytes);
        }
        Ok((1, buf[offset]))
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for bool {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::parse_u32(&buf[offset..], byteorder)?;
        match val {
            0 => Ok((bytes + padding, false)),
            1 => Ok((bytes + padding, true)),
            _ => Err(crate::wire::unmarshal::Error::InvalidBoolean),
        }
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for &'r str {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::unmarshal_str(byteorder, &buf[offset..])?;
        Ok((bytes + padding, val))
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for String {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::unmarshal_string(byteorder, &buf[offset..])?;
        Ok((bytes + padding, val))
    }
}

impl<E: Signature> Signature for Vec<E> {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            E::signature(),
        )))
    }
    fn alignment() -> usize {
        4
    }
}

impl<'r, 'buf: 'r, E: Unmarshal<'r, 'buf>> Unmarshal<'r, 'buf> for Vec<E> {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(4, buf, offset)?;
        let offset = offset + padding;
        let (_, bytes_in_array) = util::parse_u32(&buf[offset..], byteorder)?;
        let offset = offset + 4;

        let first_elem_padding = util::align_offset(E::alignment(), buf, offset)?;
        let offset = offset + first_elem_padding;

        let mut elements = Vec::new();
        let mut bytes_used_counter = 0;
        while bytes_used_counter < bytes_in_array as usize {
            let elem_padding =
                util::align_offset(E::alignment(), buf, offset + bytes_used_counter)?;
            bytes_used_counter += elem_padding;
            let (bytes_used, element) = E::unmarshal(byteorder, buf, offset + bytes_used_counter)?;
            elements.push(element);
            bytes_used_counter += bytes_used;
        }

        let total_bytes_used = padding + 4 + first_elem_padding + bytes_used_counter;

        Ok((total_bytes_used, elements))
    }
}

impl<'r, 'buf: 'r, K: Unmarshal<'r, 'buf> + std::hash::Hash + Eq, V: Unmarshal<'r, 'buf>>
    Unmarshal<'r, 'buf> for std::collections::HashMap<K, V>
{
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(4, buf, offset)?;
        let offset = offset + padding;
        let (_, bytes_in_array) = util::parse_u32(&buf[offset..], byteorder)?;
        let offset = offset + 4;

        let first_elem_padding = util::align_offset(8, buf, offset)?;
        let offset = offset + first_elem_padding;

        let mut map = std::collections::HashMap::new();
        let mut bytes_used_counter = 0;
        while bytes_used_counter < bytes_in_array as usize {
            let elem_padding = util::align_offset(8, buf, offset + bytes_used_counter)?;
            bytes_used_counter += elem_padding;

            let (key_bytes_used, key) = K::unmarshal(byteorder, buf, offset + bytes_used_counter)?;
            bytes_used_counter += key_bytes_used;

            let val_padding = util::align_offset(V::alignment(), buf, offset + bytes_used_counter)?;
            bytes_used_counter += val_padding;

            let (val_bytes_used, val) = V::unmarshal(byteorder, buf, offset + bytes_used_counter)?;
            bytes_used_counter += val_bytes_used;

            map.insert(key, val);
        }

        let total_bytes_used = padding + 4 + first_elem_padding + bytes_used_counter;

        Ok((total_bytes_used, map))
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for crate::wire::marshal_trait::UnixFd {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, val) = u32::unmarshal(byteorder, buf, offset)?;
        Ok((bytes, crate::wire::marshal_trait::UnixFd(val)))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for crate::wire::marshal_trait::SignatureWrapper<'r> {
    fn unmarshal(
        _byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::unmarshal_signature(&buf[offset..])?;
        let sig = crate::wire::marshal_trait::SignatureWrapper::new(val)
            .map_err(|_err| crate::wire::unmarshal::Error::InvalidSignature)?;
        Ok((bytes, sig))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for crate::wire::marshal_trait::ObjectPath<'r> {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, val) = <&str as Unmarshal>::unmarshal(byteorder, buf, offset)?;
        let path = crate::wire::marshal_trait::ObjectPath::new(val)
            .map_err(|_err| crate::wire::unmarshal::Error::InvalidSignature)?;
        Ok((bytes, path))
    }
}

#[test]
fn test_unmarshal_trait() {
    use crate::Marshal;

    let mut buf = Vec::new();
    let original = &["a", "b"];
    original.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();

    let (_, v) = Vec::<&str>::unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap();

    assert_eq!(original, v.as_slice());

    buf.clear();

    let mut original = std::collections::HashMap::new();
    original.insert(0u64, "abc");
    original.insert(1u64, "dce");
    original.insert(2u64, "fgh");

    original.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();

    let (_, map) =
        std::collections::HashMap::<u64, &str>::unmarshal(ByteOrder::LittleEndian, &buf, 0)
            .unwrap();
    assert_eq!(original, map);

    buf.clear();

    let orig = (0u8, true, 100u8, -123i32);
    orig.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();
    type ST = (u8, bool, u8, i32);
    let s = ST::unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap().1;
    assert_eq!(orig, s);

    buf.clear();

    use crate::wire::marshal_trait::{ObjectPath, SignatureWrapper, UnixFd};
    let orig = (
        ObjectPath::new("/a/b/c").unwrap(),
        SignatureWrapper::new("ss(aiau)").unwrap(),
        UnixFd(10),
    );
    orig.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();
    assert_eq!(
        &buf,
        &[
            6, 0, 0, 0, b'/', b'a', b'/', b'b', b'/', b'c', 0, 8, b's', b's', b'(', b'a', b'i',
            b'a', b'u', b')', 0, 0, 0, 0, 10, 0, 0, 0
        ]
    );
    let (_, (p, s, fd)) = <(ObjectPath, SignatureWrapper, UnixFd) as Unmarshal>::unmarshal(
        ByteOrder::LittleEndian,
        &buf,
        0,
    )
    .unwrap();

    assert_eq!(p.as_ref(), "/a/b/c");
    assert_eq!(s.as_ref(), "ss(aiau)");
    assert_eq!(fd.0, 10);
}
