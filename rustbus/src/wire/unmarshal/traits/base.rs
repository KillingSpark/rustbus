//! This contains the implementations for the `Unmarshal` trait for base types like integers and strings

use crate::wire::errors::UnmarshalError;
use crate::wire::unmarshal;
use crate::wire::unmarshal_context::UnmarshalContext;
use crate::wire::ObjectPath;
use crate::wire::SignatureWrapper;
use crate::Unmarshal;

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_u64()
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u32 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_u32()
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u16 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_u16()
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for i64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_i64()
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for i32 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_i32()
    }
}
impl<'buf, 'fds> Unmarshal<'buf, 'fds> for i16 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_i16()
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for u8 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_u8()
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for bool {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let val = ctx.read_u32()?;
        match val {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(UnmarshalError::InvalidBoolean),
        }
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for f64 {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let val = ctx.read_u64()?;
        Ok(f64::from_bits(val))
    }
}

impl<'buf> Unmarshal<'buf, '_> for &'buf str {
    fn unmarshal(ctx: &mut UnmarshalContext<'_, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_str()
    }
}

impl<'buf, 'fds> Unmarshal<'buf, 'fds> for String {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        ctx.read_str().map(|val| val.to_owned())
    }
}

impl<'buf, 'fds, S: AsRef<str> + From<&'buf str> + Unmarshal<'buf, 'fds>> Unmarshal<'buf, 'fds>
    for SignatureWrapper<S>
{
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let val = ctx.read_signature()?;
        let sig = SignatureWrapper::new(val.into())?;
        Ok(sig)
    }
}

impl<'buf, 'fds, S: AsRef<str> + Unmarshal<'buf, 'fds>> Unmarshal<'buf, 'fds> for ObjectPath<S> {
    fn unmarshal(ctx: &mut UnmarshalContext<'fds, 'buf>) -> unmarshal::UnmarshalResult<Self> {
        let val = <S as Unmarshal>::unmarshal(ctx)?;
        let path = ObjectPath::new(val)?;
        Ok(path)
    }
}
