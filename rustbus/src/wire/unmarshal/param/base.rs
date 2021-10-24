//! Unmarshal base params from raw bytes

use crate::params;
use crate::signature;
use crate::wire::errors::UnmarshalError;
use crate::wire::unmarshal::UnmarshalContext;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::util::*;

pub fn unmarshal_base<'a>(
    typ: signature::Base,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Base<'a>> {
    let padding = ctx.align_to(typ.get_alignment())?;

    let (bytes, param) = match typ {
        signature::Base::Byte => {
            if ctx.offset >= ctx.buf.len() {
                return Err(UnmarshalError::NotEnoughBytes);
            }
            Ok((1, params::Base::Byte(ctx.buf[ctx.offset])))
        }
        signature::Base::Uint16 => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u16(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Uint16(val)))
        }
        signature::Base::Int16 => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u16(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Int16(val as i16)))
        }
        signature::Base::Uint32 => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u32(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Uint32(val)))
        }
        signature::Base::UnixFd => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, idx) = parse_u32(slice, ctx.byteorder)?;
            if ctx.fds.len() <= idx as usize {
                Err(UnmarshalError::BadFdIndex(idx as usize))
            } else {
                let val = &ctx.fds[idx as usize];
                Ok((bytes, params::Base::UnixFd(val.clone())))
            }
        }
        signature::Base::Int32 => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u32(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Int32(val as i32)))
        }
        signature::Base::Uint64 => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u64(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Uint64(val)))
        }
        signature::Base::Int64 => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u64(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Int64(val as i64)))
        }
        signature::Base::Double => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u64(slice, ctx.byteorder)?;
            Ok((bytes, params::Base::Double(val)))
        }
        signature::Base::Boolean => {
            let slice = &ctx.buf[ctx.offset..];
            let (bytes, val) = parse_u32(slice, ctx.byteorder)?;
            match val {
                0 => Ok((bytes, params::Base::Boolean(false))),
                1 => Ok((bytes, params::Base::Boolean(true))),
                _ => Err(UnmarshalError::InvalidBoolean),
            }
        }
        signature::Base::String => {
            let (bytes, string) = unmarshal_string(ctx.byteorder, &ctx.buf[ctx.offset..])?;
            Ok((bytes, params::Base::String(string)))
        }
        signature::Base::ObjectPath => {
            let (bytes, string) = unmarshal_string(ctx.byteorder, &ctx.buf[ctx.offset..])?;
            crate::params::validate_object_path(&string)?;
            Ok((bytes, params::Base::ObjectPath(string)))
        }
        signature::Base::Signature => {
            let (bytes, string) = unmarshal_signature(&ctx.buf[ctx.offset..])?;
            crate::params::validate_signature(string)?;
            Ok((bytes, params::Base::Signature(string.to_owned())))
        }
    }?;
    ctx.offset += bytes;
    Ok((padding + bytes, param))
}
