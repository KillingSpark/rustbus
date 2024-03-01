//! This contains the implementations for the `Unmarshal` trait for container types like lists and dicts

use crate::signature;
use crate::wire::errors::UnmarshalError;
use crate::wire::marshal::traits::SignatureBuffer;
use crate::wire::unmarshal;
use crate::wire::unmarshal_context::UnmarshalContext;
use crate::Signature;
use crate::Unmarshal;
use std::borrow::Cow;

impl<'buf, 'fds, E1> Unmarshal<'buf, 'fds> for (E1,)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.align_to(8)?;
        let val1 = E1::unmarshal(ctx)?;
        Ok((val1,))
    }
}

impl<'buf, 'fds, E1, E2> Unmarshal<'buf, 'fds> for (E1, E2)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
    E2: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.align_to(8)?;
        let val1 = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let val2 = E2::unmarshal(ctx)?;

        Ok((val1, val2))
    }
}

impl<'buf, 'fds, E1, E2, E3> Unmarshal<'buf, 'fds> for (E1, E2, E3)
where
    E1: Unmarshal<'buf, 'fds> + Sized,
    E2: Unmarshal<'buf, 'fds> + Sized,
    E3: Unmarshal<'buf, 'fds> + Sized,
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.align_to(8)?;
        let val1 = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let val2 = E2::unmarshal(ctx)?;

        ctx.align_to(E3::alignment())?;
        let val3 = E3::unmarshal(ctx)?;

        Ok((val1, val2, val3))
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
        ctx.align_to(8)?;
        let val1 = E1::unmarshal(ctx)?;

        ctx.align_to(E2::alignment())?;
        let val2 = E2::unmarshal(ctx)?;

        ctx.align_to(E3::alignment())?;
        let val3 = E3::unmarshal(ctx)?;

        ctx.align_to(E4::alignment())?;
        let val4 = E4::unmarshal(ctx)?;

        Ok((val1, val2, val3, val4))
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
        ctx.align_to(Self::alignment())?;
        let elements = ctx.read_u8_slice()?;

        Ok(elements)
    }
}

unsafe fn unmarshal_slice<'a, 'buf, 'fds, E>(
    ctx: &'a mut UnmarshalContext<'fds, 'buf>,
) -> unmarshal::UnmarshalResult<&'a [E]>
where
    E: Unmarshal<'buf, 'fds>, //+ 'fds + 'buf
{
    let bytes_in_array = ctx.read_u32()? as usize;
    let alignment = E::alignment();
    ctx.align_to(alignment)?;

    // Check that we will have a range of complete elements
    if bytes_in_array % alignment != 0 {
        return Err(UnmarshalError::NotAllBytesUsed);
    }
    let content_slice = ctx.read_raw(bytes_in_array)?;

    // cast the slice from u8 to the target type
    let elem_cnt = bytes_in_array / alignment;
    let ptr = content_slice.as_ptr().cast::<E>();
    let slice = std::slice::from_raw_parts(ptr, elem_cnt);

    Ok(slice)
}

impl<'buf, 'fds, E: Unmarshal<'buf, 'fds> + Clone> Unmarshal<'buf, 'fds> for Cow<'buf, [E]> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        unsafe {
            if E::valid_slice(ctx.byteorder) {
                let src: &[E] = unmarshal_slice(ctx)?;
                // SAFETY: One of requirements is for valid_slice it is only valid for 'buf
                // Thus this lifetime cast is always valid
                let l_expand: &'buf [E] = std::mem::transmute(src);
                return Ok(Cow::Borrowed(l_expand));
            }
        }
        Vec::unmarshal(ctx).map(Cow::Owned)
    }
}

impl<'buf, 'fds, E: Unmarshal<'buf, 'fds>> Unmarshal<'buf, 'fds> for Vec<E> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        unsafe {
            if E::valid_slice(ctx.byteorder) {
                let src = unmarshal_slice::<E>(ctx)?;
                let mut ret = Vec::with_capacity(src.len());
                let dst = ret.as_mut_ptr();
                std::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
                ret.set_len(src.len());
                return Ok(ret);
            }
        }
        ctx.align_to(4)?;
        let bytes_in_array = u32::unmarshal(ctx)? as usize;

        ctx.align_to(E::alignment())?;

        let mut elements = Vec::new();
        let mut ctx = ctx.sub_context(bytes_in_array)?;
        while !ctx.remainder().is_empty() {
            ctx.align_to(E::alignment())?;
            let element = E::unmarshal(&mut ctx)?;
            elements.push(element);
        }

        Ok(elements)
    }
}

impl<'buf, 'fds, K: Unmarshal<'buf, 'fds> + std::hash::Hash + Eq, V: Unmarshal<'buf, 'fds>>
    Unmarshal<'buf, 'fds> for std::collections::HashMap<K, V>
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.align_to(4)?;
        let bytes_in_array = u32::unmarshal(ctx)? as usize;

        // align even if no elements are present
        ctx.align_to(8)?;

        let mut map = std::collections::HashMap::new();
        let mut ctx = ctx.sub_context(bytes_in_array)?;
        while !ctx.remainder().is_empty() {
            // Always align to 8
            ctx.align_to(8)?;
            let key = K::unmarshal(&mut ctx)?;

            //Align to value
            ctx.align_to(V::alignment())?;
            let val = V::unmarshal(&mut ctx)?;

            map.insert(key, val);
        }

        Ok(map)
    }
}

#[derive(Debug)]
pub struct Variant<'fds, 'buf> {
    pub(crate) sig: signature::Type,
    pub(crate) sub_ctx: UnmarshalContext<'fds, 'buf>,
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
        let mut ctx = self.sub_ctx;
        T::unmarshal(&mut ctx)
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
        let desc = ctx.read_signature()?;

        let mut sigs = match signature::Type::parse_description(desc) {
            Ok(sigs) => sigs,
            Err(_) => return Err(UnmarshalError::WrongSignature),
        };
        if sigs.len() != 1 {
            return Err(UnmarshalError::WrongSignature);
        }
        let sig = sigs.remove(0);

        ctx.align_to(sig.get_alignment())?;

        let val_bytes =
            crate::wire::validate_raw::validate_marshalled(ctx.byteorder, 0, ctx.remainder(), &sig)
                .map_err(|e| e.1)?;

        Ok(Variant {
            sig,
            sub_ctx: ctx.sub_context(val_bytes)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::message_builder::MarshalledMessageBody;

    #[test]
    fn todo_name() {
        let mut m = MarshalledMessageBody::new();
        m.push_param("test.interface").unwrap();
        m.push_param("test_property").unwrap();
        m.push_param(crate::wire::marshal::traits::Variant(42u8))
            .unwrap();

        eprintln!("Buffer: {:?}", m.get_buf());

        let mut parser = m.parser();
        assert_eq!(parser.get::<&str>().unwrap(), "test.interface");
        assert_eq!(parser.get::<&str>().unwrap(), "test_property");
        assert_eq!(parser.get_next_sig().unwrap(), "v");
        let variant = parser
            .get::<crate::wire::unmarshal::traits::Variant>()
            .unwrap();
        assert_eq!(variant.get::<u8>().unwrap(), 42);
    }
}
