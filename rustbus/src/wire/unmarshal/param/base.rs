//! Unmarshal base params from raw bytes

use crate::params;
use crate::signature;
use crate::wire::errors::UnmarshalError;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::unmarshal_context::UnmarshalContext;

pub fn unmarshal_base(
    typ: signature::Base,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Base<'static>> {
    match typ {
        signature::Base::Byte => {
            let (bytes, val) = ctx.read_u8()?;
            Ok((bytes, params::Base::Byte(val)))
        }
        signature::Base::Uint16 => {
            let (bytes, val) = ctx.read_u16()?;
            Ok((bytes, params::Base::Uint16(val)))
        }
        signature::Base::Int16 => {
            let (bytes, val) = ctx.read_i16()?;
            Ok((bytes, params::Base::Int16(val)))
        }
        signature::Base::Uint32 => {
            let (bytes, val) = ctx.read_u32()?;
            Ok((bytes, params::Base::Uint32(val)))
        }
        signature::Base::UnixFd => {
            let (bytes, idx) = ctx.read_u32()?;
            if ctx.fds.len() <= idx as usize {
                Err(UnmarshalError::BadFdIndex(idx as usize))
            } else {
                let val = &ctx.fds[idx as usize];
                Ok((bytes, params::Base::UnixFd(val.clone())))
            }
        }
        signature::Base::Int32 => {
            let (bytes, val) = ctx.read_i32()?;
            Ok((bytes, params::Base::Int32(val)))
        }
        signature::Base::Uint64 => {
            let (bytes, val) = ctx.read_u64()?;
            Ok((bytes, params::Base::Uint64(val)))
        }
        signature::Base::Int64 => {
            let (bytes, val) = ctx.read_i64()?;
            Ok((bytes, params::Base::Int64(val)))
        }
        signature::Base::Double => {
            let (bytes, val) = ctx.read_u64()?;
            Ok((bytes, params::Base::Double(val)))
        }
        signature::Base::Boolean => {
            let (bytes, val) = ctx.read_u32()?;
            match val {
                0 => Ok((bytes, params::Base::Boolean(false))),
                1 => Ok((bytes, params::Base::Boolean(true))),
                _ => Err(UnmarshalError::InvalidBoolean),
            }
        }
        signature::Base::String => {
            let (bytes, string) = ctx.read_str()?;
            Ok((bytes, params::Base::String(string.into())))
        }
        signature::Base::ObjectPath => {
            let (bytes, string) = ctx.read_str()?;
            crate::params::validate_object_path(string)?;
            Ok((bytes, params::Base::ObjectPath(string.into())))
        }
        signature::Base::Signature => {
            let (bytes, string) = ctx.read_signature()?;
            crate::params::validate_signature(string)?;
            Ok((bytes, params::Base::Signature(string.to_owned())))
        }
    }
}
