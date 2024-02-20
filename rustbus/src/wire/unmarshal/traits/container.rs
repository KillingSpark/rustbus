//! This contains the implementations for the `Unmarshal` trait for container types like lists and dicts

use crate::signature;
use crate::wire::errors::UnmarshalError;
use crate::wire::marshal::traits::SignatureBuffer;
use crate::wire::unmarshal;
use crate::wire::unmarshal::UnmarshalContext;
use crate::wire::util;
use crate::ByteOrder;
use crate::Signature;
use crate::Unmarshal;
use std::borrow::Cow;

impl<'buf, 'fds, E1> Unmarshal<'buf, 'fds> for (E1,)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(8)?;
        let (bytes, val1) = E1::unmarshal(ctx)?;
        Ok((bytes + padding, (val1,)))
    }
}

impl<'buf, 'fds, E1, E2> Unmarshal<'buf, 'fds> for (E1, E2)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
    E2: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        ctx.align_to(8)?;
        let (_bytes, val1) = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let (_bytes, val2) = E2::unmarshal(ctx)?;

        let total_bytes = ctx.offset - start_offset;
        Ok((total_bytes, (val1, val2)))
    }
}

impl<'buf, 'fds, E1, E2, E3> Unmarshal<'buf, 'fds> for (E1, E2, E3)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
    E2: Unmarshal<'buf, 'fds> + Sized,
    E3: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
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

impl<'buf, 'fds, E1, E2, E3, E4> Unmarshal<'buf, 'fds> for (E1, E2, E3, E4)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
    E2: Unmarshal<'buf, 'fds> + Sized,
    E3: Unmarshal<'buf, 'fds> + Sized,
    E4: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
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

impl<E: Signature> Signature for Vec<E> {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Array(Box::new(
            E::signature(),
        )))
    }
    #[inline]
    fn alignment() -> usize {
        <[E]>::alignment()
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        <[E]>::sig_str(s_buf)
    }
    fn has_sig(sig: &str) -> bool {
        <[E]>::has_sig(sig)
    }
}

impl<E: Signature + Clone> Signature for Cow<'_, [E]> {
    fn signature() -> crate::signature::Type {
        let e_type = Box::new(E::signature());
        crate::signature::Type::Container(crate::signature::Container::Array(e_type))
    }
    #[inline]
    fn alignment() -> usize {
        <[E]>::alignment()
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        <[E]>::sig_str(s_buf)
    }
    fn has_sig(sig: &str) -> bool {
        <[E]>::has_sig(sig)
    }
}
/// for byte arrays we can give an efficient method of decoding. This will bind the returned slice to the lifetime of the buffer.
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for &'buf [u8] {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (_, bytes_in_array) = u32::unmarshal(ctx)?;

        let elements = &ctx.buf[ctx.offset..ctx.offset + bytes_in_array as usize];
        ctx.offset += bytes_in_array as usize;

        let total_bytes_used = padding + 4 + bytes_in_array as usize;

        Ok((total_bytes_used, elements))
    }
}

unsafe fn unmarshal_slice<'a, 'buf, 'fds, E>(
    ctx: &'a mut UnmarshalContext<'fds, 'buf>,
) -> unmarshal::UnmarshalResult<&'a [E]>
where
    E: Unmarshal<'buf, 'fds>, //+ 'fds + 'buf
{
    let start_offset = ctx.offset;
    ctx.align_to(4)?;
    let (_, bytes_in_array) = u32::unmarshal(ctx)?;
    let bytes_in_array = bytes_in_array as usize;
    let alignment = E::alignment();
    ctx.align_to(alignment)?;

    // Check that we will have a range of complete elements
    if bytes_in_array % alignment != 0 {
        return Err(UnmarshalError::NotAllBytesUsed);
    }
    // Start at offset
    let start_slice = &ctx.buf[ctx.offset..];
    // Check that the buffer contains enough bytes
    if bytes_in_array > start_slice.len() {
        return Err(UnmarshalError::NotEnoughBytes);
    }
    // limit the source slice to assert that the memory access will be valid
    let content_slice = &start_slice[..bytes_in_array];

    // cast the slice from u8 to the target type
    let elem_cnt = bytes_in_array / alignment;
    let ptr = content_slice as *const [u8] as *const E;
    let slice = std::slice::from_raw_parts(ptr, elem_cnt);

    ctx.offset += bytes_in_array;
    Ok((ctx.offset - start_offset, slice))
}
impl<'buf, 'fds, E: Unmarshal<'buf, 'fds> + Clone> Unmarshal<'buf, 'fds> for Cow<'buf, [E]> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        unsafe {
            if E::valid_slice(ctx.byteorder) {
                let (used, src): (_, &[E]) = unmarshal_slice(ctx)?;
                // SAFETY: One of requirements is for valid_slice it is only valid for 'buf
                // Thus this lifetime cast is always valid
                let l_expand: &'buf [E] = std::mem::transmute(src);
                return Ok((used, Cow::Borrowed(l_expand)));
            }
        }
        Vec::unmarshal(ctx).map(|o| (o.0, Cow::Owned(o.1)))
    }
}
impl<'buf, 'fds, E: Unmarshal<'buf, 'fds>> Unmarshal<'buf, 'fds> for Vec<E> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        unsafe {
            if E::valid_slice(ctx.byteorder) {
                let (used, src) = unmarshal_slice::<E>(ctx)?;
                let mut ret = Vec::with_capacity(src.len());
                let dst = ret.as_mut_ptr();
                std::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
                ret.set_len(src.len());
                return Ok((used, ret));
            }
        }
        let start_offset = ctx.offset;
        ctx.align_to(4)?;
        let (_, bytes_in_array) = u32::unmarshal(ctx)?;

        ctx.align_to(E::alignment())?;

        let mut elements = Vec::new();
        let mut bytes_used_counter = 0;
        while bytes_used_counter < bytes_in_array as usize {
            if ctx.offset >= ctx.buf.len() {
                return Err(UnmarshalError::NotEnoughBytes);
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

impl<'buf, 'fds, K: Unmarshal<'buf, 'fds> + std::hash::Hash + Eq, V: Unmarshal<'buf, 'fds>>
    Unmarshal<'buf, 'fds> for std::collections::HashMap<K, V>
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        ctx.align_to(4)?;
        let (_, bytes_in_array) = u32::unmarshal(ctx)?;

        // align even if no elements are present
        ctx.align_to(8)?;

        let mut map = std::collections::HashMap::new();
        let mut bytes_used_counter = 0;
        while bytes_used_counter < bytes_in_array as usize {
            if ctx.offset >= ctx.buf.len() {
                return Err(UnmarshalError::NotEnoughBytes);
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

#[derive(Debug)]
pub struct Variant<'fds, 'buf> {
    pub(crate) sig: signature::Type,
    pub(crate) byteorder: ByteOrder,
    pub(crate) offset: usize,
    pub(crate) buf: &'buf [u8],
    pub(crate) fds: &'fds [crate::wire::UnixFd],
}
impl<'buf, 'fds> Variant<'fds, 'buf> {
    /// Get the [`Type`] of the value contained by the variant.
    ///
    /// [`Type`]: /rustbus/signature/enum.Type.html
    pub fn get_value_sig(&self) -> &signature::Type {
        &self.sig
    }

    /// Unmarshal the variant's value. This method is used in the same way as [`MessageBodyParser::get()`].
    ///
    /// [`MessageBodyParser::get()`]: /rustbus/message_builder/struct.MessageBodyParser.html#method.get
    pub fn get<T: Unmarshal<'buf, 'fds>>(&self) -> Result<T, UnmarshalError> {
        if self.sig != T::signature() {
            return Err(UnmarshalError::WrongSignature);
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
impl Signature for Variant<'_, '_> {
    fn signature() -> signature::Type {
        signature::Type::Container(signature::Container::Variant)
    }
    fn alignment() -> usize {
        Variant::signature().get_alignment()
    }
    #[inline]
    fn sig_str(s_buf: &mut SignatureBuffer) {
        s_buf.push_static("v");
    }
    fn has_sig(sig: &str) -> bool {
        sig.starts_with('v')
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for Variant<'fds, 'buf> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let start_offset = ctx.offset;
        let (sig_bytes, desc) = util::unmarshal_signature(&ctx.buf[ctx.offset..])?;
        ctx.offset += sig_bytes;

        let mut sigs = match signature::Type::parse_description(desc) {
            Ok(sigs) => sigs,
            Err(_) => return Err(UnmarshalError::WrongSignature),
        };
        if sigs.len() != 1 {
            return Err(UnmarshalError::WrongSignature);
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
