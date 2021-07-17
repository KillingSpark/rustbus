//! This contains the implementations for the `Unmarshal` trait for base types like integers and strings

use crate::wire::unmarshal;
use crate::wire::unmarshal::UnmarshalContext;
use crate::wire::util;
use crate::wire::ObjectPath;
use crate::wire::SignatureWrapper;
use crate::Signature;
use crate::Unmarshal;

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u64(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u32 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u16 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u16(&ctx.buf[ctx.offset..], ctx.byteorder)?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for i64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u64(&ctx.buf[ctx.offset..], ctx.byteorder)
            .map(|(bytes, val)| (bytes, val as i64))?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for i32 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)
            .map(|(bytes, val)| (bytes, val as i32))?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for i16 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::parse_u16(&ctx.buf[ctx.offset..], ctx.byteorder)
            .map(|(bytes, val)| (bytes, val as i16))?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u8 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        if ctx.offset >= ctx.buf.len() {
            return Err(crate::wire::unmarshal::Error::NotEnoughBytes);
        }
        let val = ctx.buf[ctx.offset];
        ctx.offset += 1;
        Ok((1, val))
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for bool {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
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

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for &'buf str {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::unmarshal_str(ctx.byteorder, &ctx.buf[ctx.offset..])?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for String {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let padding = ctx.align_to(Self::alignment())?;
        let (bytes, val) = util::unmarshal_string(ctx.byteorder, &ctx.buf[ctx.offset..])?;
        ctx.offset += bytes;
        Ok((bytes + padding, val))
    }
}

impl<'buf, 'fds, S: AsRef<str> + From<&'buf str> + Unmarshal<'buf, 'fds>> Unmarshal<'buf, 'fds>
    for SignatureWrapper<S>
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        // No alignment needed. Signature is aligned to 1
        let (bytes, val) = util::unmarshal_signature(&ctx.buf[ctx.offset..])?;
        ctx.offset += bytes;
        let sig = SignatureWrapper::new(val.into())?;
        Ok((bytes, sig))
    }
}

impl<'buf, 'fds, S: AsRef<str> + Unmarshal<'buf, 'fds>> Unmarshal<'buf, 'fds> for ObjectPath<S> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, val) = <S as Unmarshal>::unmarshal(ctx)?;
        let path = ObjectPath::new(val)?;
        Ok((bytes, path))
    }
}
