//! Provides the Unmarshal trait and the implementations for the base types

use crate::wire::marshal::traits::Signature;
use crate::wire::unmarshal;
use crate::wire::util;
use crate::ByteOrder;

/// This trait has to be supported to get parameters ergonomically out of a MarshalledMessage.
/// There are implementations for the base types, Vecs, Hashmaps, and tuples of up to 5 elements
/// if the contained types are Unmarshal.
/// If you deal with basic messages, this should cover all your needs and you dont need to implement this type for
/// your own types.
///
/// # Implementing for your own structs
/// You can of course add your own implementations for you your own types.
/// For this to work properly the signature must be correct and you need to report all bytes you consumed
/// in the T::unmarshal(...) call. THIS INCLUDES PADDING.
///
/// Typically your code should look like this, keeping track of all padding and adjusting the local offset appropriatly
/// ```rust
/// struct MyStruct{ mycoolint: u64}
/// use rustbus::wire::marshal::traits::Signature;
/// use rustbus::signature;
/// impl Signature for MyStruct {
///     fn signature() -> signature::Type {
///         signature::Type::Container(signature::Container::Struct(vec![
///             u64::signature(),
///         ]))
///     }
///
///     fn alignment() -> usize {
///         8
///     }
/// }  
/// use rustbus::wire::unmarshal::traits::Unmarshal;
/// use rustbus::wire::unmarshal::UnmarshalResult;
/// use rustbus::wire::util;
/// use rustbus::ByteOrder;
/// impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for MyStruct {
///     fn unmarshal(
///         byteorder: ByteOrder,
///         buf: &'buf [u8],
///         offset: usize,
///     ) -> UnmarshalResult<Self> {
///         // check that we are aligned properly
///         let padding = util::align_offset(Self::alignment(), buf, offset)?;
///         let offset = offset + padding;
///
///         // decode some stuff and adjust offset
///         let (bytes, mycoolint) = u64::unmarshal(byteorder, buf, offset)?;
///         let offset = offset + bytes;
///         
///         // some more decoding if the struct had more fields
///         // let padding2 = util::align_offset(u64::alignment(), buf, offset)?;
///         // let offset = offset + padding2;
///         // ect, etc
///         Ok((padding + bytes /* + padding2  + more_bytes + ...*/, MyStruct{mycoolint}))
///     }
/// }
/// ```
///
/// This is of course just an example, this could be solved by using
/// ```rust,ignore
/// let (bytes, mycoolint) =  <(u64,) as Unmarshal>::unmarshal(...)
/// ```
///
/// ## Cool things you can do
/// If the message contains some form of secondary marshalling, of another format, you can do this here too, insteadof copying the bytes
/// array around before doing the secondary unmarshalling. Just keep in mind that you have to report the accurat number of bytes used, and not to
/// use any bytes in the message, not belonging to that byte array
/// ```rust,ignore
/// impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for MyStruct {
///     fn unmarshal(
///         byteorder: ByteOrder,
///         buf: &'buf [u8],
///         offset: usize,
///     ) -> UnmarshalResult<Self> {
///         // check that we are aligned properly
///         let padding = util::align_offset(Self::alignment(), buf, offset)?;
///         let offset = offset + padding;
///
///         // decode array length stuff and adjust offset
///         let (bytes, arraylen) = u32::unmarshal(byteorder, buf, offset)?;
///         let offset = offset + bytes;
///         let marshalled_stuff = &buf[offset..arraylen as usize];
///         let unmarshalled_stuff = external_crate::unmarshal_stuff(&buf);
///         Ok((padding + bytes + arraylen as usize, MyStruct{unmarshalled_stuff}))
///     }
/// }
/// ```

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

/// for byte arrays we can give an efficient method of decoding. This will bind the returned slice to the lifetime of the buffer.
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for &'r [u8] {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(4, buf, offset)?;
        let offset = offset + padding;
        let (_, bytes_in_array) = util::parse_u32(&buf[offset..], byteorder)?;
        let offset = offset + 4;

        let first_elem_padding = util::align_offset(u8::alignment(), buf, offset)?;
        let offset = offset + first_elem_padding;

        let elements = &buf[offset..offset + bytes_in_array as usize];

        let total_bytes_used = padding + 4 + bytes_in_array as usize;

        Ok((total_bytes_used, elements))
    }
}

#[test]
fn test_unmarshal_byte_array() {
    use crate::Marshal;
    let mut orig = vec![];
    for x in 0..1024 {
        orig.push((x % 255) as u8);
    }

    let mut buf = Vec::new();
    orig.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();
    assert_eq!(&buf[..4], &[0, 4, 0, 0]);
    assert_eq!(buf.len(), 1028);
    let (bytes, unorig) =
        <&[u8] as Unmarshal>::unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap();
    assert_eq!(bytes, orig.len() + 4);
    assert_eq!(orig, unorig);

    // even slices of slices of u8 work efficiently
    let mut orig1 = vec![];
    let mut orig2 = vec![];
    for x in 0..1024 {
        orig1.push((x % 255) as u8);
    }
    for x in 0..1024 {
        orig2.push(((x + 4) % 255) as u8);
    }

    let orig = vec![orig1.as_slice(), orig2.as_slice()];

    let mut buf = Vec::new();
    orig.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();

    // unorig[x] points into the appropriate region in buf, and unorigs lifetime is bound to buf
    let (_bytes, unorig) =
        <Vec<&[u8]> as Unmarshal>::unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap();
    assert_eq!(orig, unorig);
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

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for crate::wire::marshal::traits::UnixFd {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, val) = u32::unmarshal(byteorder, buf, offset)?;
        Ok((bytes, crate::wire::marshal::traits::UnixFd(val)))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for crate::wire::marshal::traits::SignatureWrapper<'r> {
    fn unmarshal(
        _byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(Self::alignment(), buf, offset)?;
        let offset = offset + padding;
        let (bytes, val) = util::unmarshal_signature(&buf[offset..])?;
        let sig = crate::wire::marshal::traits::SignatureWrapper::new(val)?;
        Ok((bytes, sig))
    }
}
impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for crate::wire::marshal::traits::ObjectPath<'r> {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, val) = <&str as Unmarshal>::unmarshal(byteorder, buf, offset)?;
        let path = crate::wire::marshal::traits::ObjectPath::new(val)?;
        Ok((bytes, path))
    }
}

#[test]
fn test_unmarshal_traits() {
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

    use crate::wire::marshal::traits::{ObjectPath, SignatureWrapper, UnixFd};
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
