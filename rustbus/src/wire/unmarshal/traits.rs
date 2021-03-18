//! Provides the Unmarshal trait and the implementations for the base types

use crate::signature;
use crate::wire::marshal::traits::Signature;
use crate::wire::unmarshal;
use crate::wire::unmarshal::UnmarshalContext;
use crate::wire::util;
use crate::ByteOrder;

/// This trait has to be supported to get parameters ergonomically out of a MarshalledMessage.
/// There are implementations for the base types, Vecs, Hashmaps, and tuples of up to 5 elements
/// if the contained types are Unmarshal.
/// If you deal with basic messages, this should cover all your needs and you dont need to implement this type for
/// your own types.
///
/// There is a crate (rustbus_derive) for deriving Unmarshal impls with #[derive(rustbus_derive::Marshal)]. This should work for most of your needs.
/// You can of course derive Signature as well.
///
/// If there are special needs, you can implement Unmarshal for your own structs:
///
/// # Implementing for your own structs
/// You can of course add your own implementations for types.
/// For this to work properly the signature must be correct and you need to report all bytes you consumed
/// in the T::unmarshal(...) call. THIS INCLUDES PADDING.
///
/// Typically your code should look like this:
/// ```rust
/// struct MyStruct{ mycoolint: u64}
/// use rustbus::wire::marshal::traits::Signature;
/// use rustbus::wire::unmarshal::UnmarshalContext;
/// use rustbus::wire::unmarshal;
/// use rustbus::signature;
/// impl Signature for MyStruct {
///     fn signature() -> signature::Type {
///         signature::Type::Container(signature::Container::Struct(signature::StructTypes::new(vec![
///             u64::signature(),
///         ]).unwrap()))
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
/// impl<'r> Unmarshal<'r> for MyStruct {
///    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
///         let start_offset = ctx.offset;
///         // check that we are aligned properly!
///         // This is necessary at the start of each struct! They need to be aligned to 8 bytes!
///         let padding = ctx.align_to(Self::alignment())?;
///
///         // decode some stuff and adjust offset
///         let (bytes, mycoolint) = u64::unmarshal(ctx)?;
///         
///         // some more decoding if the struct had more fields
///         // ....
///         
///         //then report the total bytes used by unmarshalling this type (INCLUDING padding at the beginning!):
///         let total_bytes = ctx.offset - start_offset;
///         Ok((total_bytes, MyStruct{mycoolint}))
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
/// If the message contains some form of secondary marshalling, of another format, you can do this here too, instead of copying the bytes
/// array around before doing the secondary unmarshalling. Just keep in mind that you have to report the accurate number of bytes used, and not to
/// use any bytes in the message, not belonging to that byte array
///
/// As an example, lets assume your message contains a byte-array that is actually json data. Then you can use serde_json to unmarshal that array
/// directly here without having to do a separate step for that.
/// ```rust,ignore
/// impl<'r> Unmarshal<'r> for MyStruct {
///    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
///         let start_offset = ctx.offset;
///         // check that we are aligned properly
///         let padding = ctx.align_to(Self::alignment())?;
///
///         // get the slice that contains marshalled data, and unmarshal it directly here!
///         let (bytes, raw_data) = <&[u8] as Unmarshal>::unmarshal(ctx)?;
///         let unmarshalled_stuff = external_crate::unmarshal_stuff(&raw_data);
///
///         //then report the total bytes used by unmarshalling this type (INCLUDING padding at the beginning!):
///         let total_bytes = ctx.offset - start_offset;
///         Ok((total_bytes, MyStruct{unmarshalled_stuff}))
///     }
/// }
/// ```

pub trait Unmarshal<'r>: Sized + Signature {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self>;
}

pub fn unmarshal<'r, T: Unmarshal<'r>>(
    ctx: &mut UnmarshalContext<'r>,
) -> unmarshal::UnmarshalResult<T> {
    T::unmarshal(ctx)
}

#[test]
fn test_generic_unmarshal() {
    use crate::wire::marshal::MarshalContext;
    use crate::Marshal;

    let mut fds = Vec::new();
    let mut buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // annotate the receiver with a type &str to unmarshal a &str
    "ABCD".marshal(ctx).unwrap();
    let _s: &str = unmarshal(&mut UnmarshalContext {
        buf: &ctx.buf,
        byteorder: ctx.byteorder,
        fds: &ctx.fds,
        offset: 0,
    })
    .unwrap()
    .1;

    // annotate the receiver with a type bool to unmarshal a bool
    ctx.buf.clear();
    true.marshal(ctx).unwrap();
    let _b: bool = unmarshal(&mut UnmarshalContext {
        buf: &ctx.buf,
        byteorder: ctx.byteorder,
        fds: &ctx.fds,
        offset: 0,
    })
    .unwrap()
    .1;

    // can also use turbofish syntax
    ctx.buf.clear();
    0i32.marshal(ctx).unwrap();
    let _i = unmarshal::<i32>(&mut UnmarshalContext {
        buf: &ctx.buf,
        byteorder: ctx.byteorder,
        fds: &ctx.fds,
        offset: 0,
    })
    .unwrap()
    .1;

    // No type info on let arg = unmarshal(...) is needed if it can be derived by other means
    ctx.buf.clear();
    fn x(_arg: (i32, i32, &str)) {};
    (0, 0, "ABCD").marshal(ctx).unwrap();
    let arg = unmarshal(&mut UnmarshalContext {
        buf: &ctx.buf,
        byteorder: ctx.byteorder,
        fds: &ctx.fds,
        offset: 0,
    })
    .unwrap()
    .1;
    x(arg);
}

impl<'r, E1> Unmarshal<'r> for (E1,)
where
    E1: Unmarshal<'r> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(8)?;
        let (bytes, val1) = E1::unmarshal(ctx)?;
        Ok((bytes + padding, (val1,)))
    }
}

impl<'r, E1, E2> Unmarshal<'r> for (E1, E2)
where
    E1: Unmarshal<'r> + Sized,
    E2: Unmarshal<'r> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        ctx.align_to(8)?;
        let (_bytes, val1) = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let (_bytes, val2) = E2::unmarshal(ctx)?;

        let total_bytes = ctx.offset - start_offset;
        Ok((total_bytes, (val1, val2)))
    }
}

impl<'r, E1, E2, E3> Unmarshal<'r> for (E1, E2, E3)
where
    E1: Unmarshal<'r> + Sized,
    E2: Unmarshal<'r> + Sized,
    E3: Unmarshal<'r> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;

        ctx.align_to(8)?;
        let (_bytes, val1) = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let (_bytes, val2) = E2::unmarshal(ctx)?;

        ctx.align_to(E3::alignment())?;
        let (_bytes, val3) = E3::unmarshal(ctx)?;

        let total_bytes = ctx.offset - start_offset;
        Ok((total_bytes, (val1, val2, val3)))
    }
}

impl<'r, E1, E2, E3, E4> Unmarshal<'r> for (E1, E2, E3, E4)
where
    E1: Unmarshal<'r> + Sized,
    E2: Unmarshal<'r> + Sized,
    E3: Unmarshal<'r> + Sized,
    E4: Unmarshal<'r> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;

        ctx.align_to(8)?;
        let (_bytes, val1) = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let (_bytes, val2) = E2::unmarshal(ctx)?;

        ctx.align_to(E3::alignment())?;
        let (_bytes, val3) = E3::unmarshal(ctx)?;

        ctx.align_to(E4::alignment())?;
        let (_bytes, val4) = E4::unmarshal(ctx)?;

        let total_bytes = ctx.offset - start_offset;
        Ok((total_bytes, (val1, val2, val3, val4)))
    }
}

impl<'r> Unmarshal<'r> for u64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u64(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'r> Unmarshal<'r> for u32 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'r> Unmarshal<'r> for u16 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u16(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'r> Unmarshal<'r> for i64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u64(&ctx.buf[ctx.offset..], ctx.byteorder)
            .map(|(bytes, val)| (bytes, val as i64))?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'r> Unmarshal<'r> for i32 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)
            .map(|(bytes, val)| (bytes, val as i32))?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'r> Unmarshal<'r> for i16 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u16(&ctx.buf[ctx.offset..], ctx.byteorder)
            .map(|(bytes, val)| (bytes, val as i16))?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}

impl<'r> Unmarshal<'r> for u8 {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        if ctx.buf[ctx.offset..].is_empty() {
            return Err(crate::wire::unmarshal::Error::NotEnoughBytes);
        }
        let val = ctx.buf[ctx.offset];
        ctx.offset += 1;
        Ok((1, val))
    }
}

impl<'r> Unmarshal<'r> for bool {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        match val {
            0 => Ok((bytes + padding, false)),
            1 => Ok((bytes + padding, true)),
            _ => Err(crate::wire::unmarshal::Error::InvalidBoolean),
        }
    }
}

impl<'r> Unmarshal<'r> for &'r str {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::unmarshal_str(ctx.byteorder, &ctx.buf[ctx.offset..])?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}

impl<'r> Unmarshal<'r> for String {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::unmarshal_string(ctx.byteorder, &ctx.buf[ctx.offset..])?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}

/// for byte arrays we can give an efficient method of decoding. This will bind the returned slice to the lifetime of the buffer.
impl<'r> Unmarshal<'r> for &'r [u8] {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (_, bytes_in_array) = u32::unmarshal(ctx)?;

        let elements = &ctx.buf[ctx.offset..ctx.offset + bytes_in_array as usize];
        ctx.offset += bytes_in_array as usize;

        let total_bytes_used = padding + 4 + bytes_in_array as usize;

        Ok((total_bytes_used, elements))
    }
}

#[test]
fn test_unmarshal_byte_array() {
    use crate::wire::marshal::MarshalContext;
    use crate::Marshal;

    let mut orig = vec![];
    for x in 0..1024 {
        orig.push((x % 255) as u8);
    }

    let mut fds = Vec::new();
    let mut buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    orig.marshal(ctx).unwrap();
    assert_eq!(&ctx.buf[..4], &[0, 4, 0, 0]);
    assert_eq!(ctx.buf.len(), 1028);
    let (bytes, unorig) = <&[u8] as Unmarshal>::unmarshal(&mut UnmarshalContext {
        buf: ctx.buf,
        fds: ctx.fds,
        byteorder: ctx.byteorder,
        offset: 0,
    })
    .unwrap();
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

    ctx.buf.clear();
    orig.marshal(ctx).unwrap();

    // unorig[x] points into the appropriate region in buf, and unorigs lifetime is bound to buf
    let (_bytes, unorig) = <Vec<&[u8]> as Unmarshal>::unmarshal(&mut UnmarshalContext {
        buf: ctx.buf,
        fds: ctx.fds,
        byteorder: ctx.byteorder,
        offset: 0,
    })
    .unwrap();
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

impl<'r, E: Unmarshal<'r>> Unmarshal<'r> for Vec<E> {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        ctx.align_to(4)?;
        let (_, bytes_in_array) = u32::unmarshal(ctx)?;

        ctx.align_to(E::alignment())?;

        let mut elements = Vec::new();
        let mut bytes_used_counter = 0;
        while bytes_used_counter < bytes_in_array as usize {
            if ctx.offset >= ctx.buf.len() {
                return Err(super::Error::NotEnoughBytes);
            }

            let elem_padding = util::align_offset(E::alignment(), ctx.buf, ctx.offset)?;

            bytes_used_counter += elem_padding;
            ctx.offset += elem_padding;

            let (bytes_used, element) = E::unmarshal(ctx)?;
            elements.push(element);
            bytes_used_counter += bytes_used;
        }

        let total_bytes_used = ctx.offset - start_offset;

        Ok((total_bytes_used, elements))
    }
}

impl<'r, 'buf: 'r, 'fds, K: Unmarshal<'r> + std::hash::Hash + Eq, V: Unmarshal<'r>> Unmarshal<'r>
    for std::collections::HashMap<K, V>
{
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        ctx.align_to(4)?;
        let (_, bytes_in_array) = u32::unmarshal(ctx)?;

        // align even if no elements are present
        ctx.align_to(8)?;

        let mut map = std::collections::HashMap::new();
        let mut bytes_used_counter = 0;
        while bytes_used_counter < bytes_in_array as usize {
            if ctx.offset >= ctx.buf.len() {
                return Err(super::Error::NotEnoughBytes);
            }

            let elem_padding = util::align_offset(8, ctx.buf, ctx.offset)?;
            bytes_used_counter += elem_padding;
            ctx.offset += elem_padding;

            let (key_bytes_used, key) = K::unmarshal(ctx)?;
            bytes_used_counter += key_bytes_used;

            let val_padding = util::align_offset(V::alignment(), ctx.buf, ctx.offset)?;
            bytes_used_counter += val_padding;
            ctx.offset += val_padding;

            let (val_bytes_used, val) = V::unmarshal(ctx)?;
            bytes_used_counter += val_bytes_used;

            map.insert(key, val);
        }

        let total_bytes_used = ctx.offset - start_offset;

        Ok((total_bytes_used, map))
    }
}

impl<'r> Unmarshal<'r> for crate::wire::marshal::traits::SignatureWrapper<'r> {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        // No alignment needed. Signature is aligned to 1
        let (bytes, val) = util::unmarshal_signature(&ctx.buf[ctx.offset..])?;
        ctx.offset += bytes;
        let sig = crate::wire::marshal::traits::SignatureWrapper::new(val)?;
        Ok((bytes, sig))
    }
}
impl<'r, S: AsRef<str> + Unmarshal<'r>> Unmarshal<'r>
    for crate::wire::marshal::traits::ObjectPath<S>
{
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, val) = <S as Unmarshal>::unmarshal(ctx)?;
        let path = crate::wire::marshal::traits::ObjectPath::new(val)?;
        Ok((bytes, path))
    }
}

#[derive(Debug)]
pub struct Variant<'buf> {
    pub(crate) sig: signature::Type,
    pub(crate) byteorder: ByteOrder,
    pub(crate) offset: usize,
    pub(crate) buf: &'buf [u8],
    pub(crate) fds: &'buf [crate::wire::UnixFd],
}
impl<'r> Variant<'r> {
    /// Get the [`Type`] of the value contained by the variant.
    ///
    /// [`Type`]: /rustbus/signature/enum.Type.html
    pub fn get_value_sig(&self) -> &signature::Type {
        &self.sig
    }

    /// Unmarshal the variant's value. This method is used in the same way as [`MessageBodyParser::get()`].
    ///
    /// [`MessageBodyParser::get()`]: /rustbus/message_builder/struct.MessageBodyParser.html#method.get
    pub fn get<T: Unmarshal<'r>>(&self) -> Result<T, unmarshal::Error> {
        if self.sig != T::signature() {
            return Err(unmarshal::Error::WrongSignature);
        }
        let mut ctx = UnmarshalContext {
            byteorder: self.byteorder,
            offset: self.offset,
            buf: self.buf,
            fds: self.fds,
        };
        T::unmarshal(&mut ctx).map(|r| r.1)
    }
}
impl Signature for Variant<'_> {
    fn signature() -> signature::Type {
        signature::Type::Container(signature::Container::Variant)
    }
    fn alignment() -> usize {
        Variant::signature().get_alignment()
    }
}
impl<'r> Unmarshal<'r> for Variant<'r> {
    fn unmarshal(ctx: &mut UnmarshalContext<'r>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        let (sig_bytes, desc) = util::unmarshal_signature(&ctx.buf[ctx.offset..])?;
        ctx.offset += sig_bytes;

        let mut sigs = match signature::Type::parse_description(desc) {
            Ok(sigs) => sigs,
            Err(_) => return Err(unmarshal::Error::WrongSignature),
        };
        if sigs.len() != 1 {
            return Err(unmarshal::Error::WrongSignature);
        }
        let sig = sigs.remove(0);

        ctx.align_to(sig.get_alignment())?;

        let start_loc = ctx.offset;

        let val_bytes = crate::wire::validate_raw::validate_marshalled(
            ctx.byteorder,
            ctx.offset,
            ctx.buf,
            &sig,
        )
        .map_err(|e| e.1)?;
        ctx.offset += val_bytes;

        let total_bytes = ctx.offset - start_offset;
        Ok((
            total_bytes,
            Variant {
                sig,
                buf: &ctx.buf[..ctx.offset],
                offset: start_loc,
                byteorder: ctx.byteorder,
                fds: ctx.fds,
            },
        ))
    }
}
#[test]
fn test_unmarshal_traits() {
    use crate::wire::marshal::MarshalContext;
    use crate::Marshal;

    let mut fds = Vec::new();
    let mut buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    let original = &["a", "b"];
    original.marshal(ctx).unwrap();

    let (_, v) = Vec::<&str>::unmarshal(&mut UnmarshalContext {
        buf: ctx.buf,
        fds: ctx.fds,
        byteorder: ctx.byteorder,
        offset: 0,
    })
    .unwrap();

    assert_eq!(original, v.as_slice());

    ctx.buf.clear();

    let mut original = std::collections::HashMap::new();
    original.insert(0u64, "abc");
    original.insert(1u64, "dce");
    original.insert(2u64, "fgh");

    original.marshal(ctx).unwrap();

    let (_, map) = std::collections::HashMap::<u64, &str>::unmarshal(&mut UnmarshalContext {
        buf: ctx.buf,
        fds: ctx.fds,
        byteorder: ctx.byteorder,
        offset: 0,
    })
    .unwrap();
    assert_eq!(original, map);

    ctx.buf.clear();

    let orig = (30u8, true, 100u8, -123i32);
    orig.marshal(ctx).unwrap();
    type ST = (u8, bool, u8, i32);
    let s = ST::unmarshal(&mut UnmarshalContext {
        buf: ctx.buf,
        fds: ctx.fds,
        byteorder: ctx.byteorder,
        offset: 0,
    })
    .unwrap()
    .1;
    assert_eq!(orig, s);

    ctx.buf.clear();

    use crate::wire::marshal::traits::{ObjectPath, SignatureWrapper};
    use crate::wire::UnixFd;
    let orig_fd = UnixFd::new(nix::unistd::dup(1).unwrap());
    let orig = (
        ObjectPath::new("/a/b/c").unwrap(),
        SignatureWrapper::new("ss(aiau)").unwrap(),
        &orig_fd,
    );
    orig.marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[
            6, 0, 0, 0, b'/', b'a', b'/', b'b', b'/', b'c', 0, 8, b's', b's', b'(', b'a', b'i',
            b'a', b'u', b')', 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );
    let (_, (p, s, _fd)) =
        <(ObjectPath<String>, SignatureWrapper, UnixFd) as Unmarshal>::unmarshal(
            &mut UnmarshalContext {
                buf: ctx.buf,
                fds: ctx.fds,
                byteorder: ctx.byteorder,
                offset: 0,
            },
        )
        .unwrap();

    assert_eq!(p.as_ref(), "/a/b/c");
    assert_eq!(s.as_ref(), "ss(aiau)");
}

#[test]
fn test_variant() {
    use crate::message_builder::MarshalledMessageBody;
    use crate::params::{Array, Base, Container, Dict, Param, Variant as ParamVariant};
    use crate::signature::Type;
    use crate::wire::marshal::traits::SignatureWrapper;
    use std::collections::HashMap;

    // inital test data
    let params: [(Param, Type); 10] = [
        (Base::Byte(0x41).into(), u8::signature()),
        (Base::Int16(-1234).into(), i16::signature()),
        (Base::Uint16(1234).into(), u16::signature()),
        (Base::Int32(-1234567).into(), i32::signature()),
        (Base::Uint32(1234567).into(), u32::signature()),
        (Base::Int64(-1234568901234).into(), i64::signature()),
        (Base::Uint64(1234568901234).into(), u64::signature()),
        (
            Base::String("Hello world!".to_string()).into(),
            String::signature(),
        ),
        (
            Base::Signature("sy".to_string()).into(),
            SignatureWrapper::signature(),
        ),
        (Base::Boolean(true).into(), bool::signature()),
    ];

    // push initial data as individual variants
    let mut body = MarshalledMessageBody::new();
    for param in &params {
        let cont = Container::Variant(Box::new(ParamVariant {
            sig: param.1.clone(),
            value: param.0.clone(),
        }));
        body.push_old_param(&Param::Container(cont)).unwrap();
    }

    // push initial data as Array of variants
    let var_vec = params
        .iter()
        .map(|(param, typ)| {
            Param::Container(Container::Variant(Box::new(ParamVariant {
                sig: typ.clone(),
                value: param.clone(),
            })))
        })
        .collect();
    let vec_param = Param::Container(Container::Array(Array {
        element_sig: Variant::signature(),
        values: var_vec,
    }));
    body.push_old_param(&vec_param).unwrap();

    // push initial data as Dict of {String,variants}
    let var_map = params
        .iter()
        .enumerate()
        .map(|(i, (param, typ))| {
            (
                Base::String(format!("{}", i)),
                Param::Container(Container::Variant(Box::new(ParamVariant {
                    sig: typ.clone(),
                    value: param.clone(),
                }))),
            )
        })
        .collect();
    let map_param = Param::Container(Container::Dict(Dict {
        key_sig: crate::signature::Base::String,
        value_sig: Variant::signature(),
        map: var_map,
    }));
    body.push_old_param(&map_param).unwrap();

    // check the individual variants
    let mut parser = body.parser();
    assert_eq!(
        0x41_u8,
        parser.get::<Variant>().unwrap().get::<u8>().unwrap()
    );
    assert_eq!(
        -1234_i16,
        parser.get::<Variant>().unwrap().get::<i16>().unwrap()
    );
    assert_eq!(
        1234_u16,
        parser.get::<Variant>().unwrap().get::<u16>().unwrap()
    );
    assert_eq!(
        -1234567_i32,
        parser.get::<Variant>().unwrap().get::<i32>().unwrap()
    );
    assert_eq!(
        1234567_u32,
        parser.get::<Variant>().unwrap().get::<u32>().unwrap()
    );
    assert_eq!(
        -1234568901234_i64,
        parser.get::<Variant>().unwrap().get::<i64>().unwrap()
    );
    assert_eq!(
        1234568901234_u64,
        parser.get::<Variant>().unwrap().get::<u64>().unwrap()
    );
    assert_eq!(
        "Hello world!",
        parser.get::<Variant>().unwrap().get::<&str>().unwrap()
    );
    assert_eq!(
        SignatureWrapper::new("sy").unwrap(),
        parser.get::<Variant>().unwrap().get().unwrap()
    );
    assert_eq!(
        true,
        parser.get::<Variant>().unwrap().get::<bool>().unwrap()
    );

    // check Array of variants
    let var_vec: Vec<Variant> = parser.get().unwrap();
    assert_eq!(0x41_u8, var_vec[0].get::<u8>().unwrap());
    assert_eq!(-1234_i16, var_vec[1].get::<i16>().unwrap());
    assert_eq!(1234_u16, var_vec[2].get::<u16>().unwrap());
    assert_eq!(-1234567_i32, var_vec[3].get::<i32>().unwrap());
    assert_eq!(1234567_u32, var_vec[4].get::<u32>().unwrap());
    assert_eq!(-1234568901234_i64, var_vec[5].get::<i64>().unwrap());
    assert_eq!(1234568901234_u64, var_vec[6].get::<u64>().unwrap());
    assert_eq!("Hello world!", var_vec[7].get::<&str>().unwrap());
    assert_eq!(
        SignatureWrapper::new("sy").unwrap(),
        var_vec[8].get().unwrap()
    );
    assert_eq!(true, var_vec[9].get::<bool>().unwrap());

    // check Dict of {String, variants}
    let var_map: HashMap<String, Variant> = parser.get().unwrap();
    assert_eq!(0x41_u8, var_map["0"].get::<u8>().unwrap());
    assert_eq!(-1234_i16, var_map["1"].get::<i16>().unwrap());
    assert_eq!(1234_u16, var_map["2"].get::<u16>().unwrap());
    assert_eq!(-1234567_i32, var_map["3"].get::<i32>().unwrap());
    assert_eq!(1234567_u32, var_map["4"].get::<u32>().unwrap());
    assert_eq!(-1234568901234_i64, var_map["5"].get::<i64>().unwrap());
    assert_eq!(1234568901234_u64, var_map["6"].get::<u64>().unwrap());
    assert_eq!("Hello world!", var_map["7"].get::<&str>().unwrap());
    assert_eq!(
        SignatureWrapper::new("sy").unwrap(),
        var_map["8"].get().unwrap()
    );
    assert_eq!(true, var_map["9"].get::<bool>().unwrap());
}
