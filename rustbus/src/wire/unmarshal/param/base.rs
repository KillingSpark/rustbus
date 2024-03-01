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
            let val = ctx.read_u8()?;
            Ok(params::Base::Byte(val))
        }
        signature::Base::Uint16 => {
            let val = ctx.read_u16()?;
            Ok(params::Base::Uint16(val))
        }
        signature::Base::Int16 => {
            let val = ctx.read_i16()?;
            Ok(params::Base::Int16(val))
        }
        signature::Base::Uint32 => {
            let val = ctx.read_u32()?;
            Ok(params::Base::Uint32(val))
        }
        signature::Base::UnixFd => {
            let val = ctx.read_unixfd()?;
            Ok(params::Base::UnixFd(val))
        }
        signature::Base::Int32 => {
            let val = ctx.read_i32()?;
            Ok(params::Base::Int32(val))
        }
        signature::Base::Uint64 => {
            let val = ctx.read_u64()?;
            Ok(params::Base::Uint64(val))
        }
        signature::Base::Int64 => {
            let val = ctx.read_i64()?;
            Ok(params::Base::Int64(val))
        }
        signature::Base::Double => {
            let val = ctx.read_u64()?;
            Ok(params::Base::Double(val))
        }
        signature::Base::Boolean => {
            let val = ctx.read_u32()?;
            match val {
                0 => Ok(params::Base::Boolean(false)),
                1 => Ok(params::Base::Boolean(true)),
                _ => Err(UnmarshalError::InvalidBoolean),
            }
        }
        signature::Base::String => {
            let string = ctx.read_str()?;
            Ok(params::Base::String(string.into()))
        }
        signature::Base::ObjectPath => {
            let string = ctx.read_str()?;
            crate::params::validate_object_path(string)?;
            Ok(params::Base::ObjectPath(string.into()))
        }
        signature::Base::Signature => {
            let string = ctx.read_signature()?;
            crate::params::validate_signature(string)?;
            Ok(params::Base::Signature(string.to_owned()))
        }
    }
}
