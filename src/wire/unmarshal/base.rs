//! Unmarshal base params from raw bytes

use crate::params;
use crate::signature;
use crate::wire::unmarshal::Error;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::util::*;
use crate::ByteOrder;

pub fn unmarshal_base<'a>(
    byteorder: ByteOrder,
    buf: &[u8],
    typ: signature::Base,
    offset: usize,
) -> UnmarshalResult<params::Base<'a>> {
    let padding = align_offset(typ.get_alignment(), buf, offset)?;

    match typ {
        signature::Base::Byte => {
            if buf.is_empty() {
                return Err(Error::NotEnoughBytes);
            }
            Ok((1, params::Base::Byte(buf[offset])))
        }
        signature::Base::Uint16 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 2];
            let (bytes, val) = parse_u16(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Uint16(val)))
        }
        signature::Base::Int16 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 2];
            let (bytes, val) = parse_u16(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Int16(val as i16)))
        }
        signature::Base::Uint32 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Uint32(val)))
        }
        signature::Base::UnixFd => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, byteorder)?;
            Ok((bytes + padding, params::Base::UnixFd(val)))
        }
        signature::Base::Int32 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Int32(val as i32)))
        }
        signature::Base::Uint64 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Uint64(val)))
        }
        signature::Base::Int64 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Int64(val as i64)))
        }
        signature::Base::Double => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, byteorder)?;
            Ok((bytes + padding, params::Base::Double(val)))
        }
        signature::Base::Boolean => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, byteorder)?;
            match val {
                0 => Ok((bytes + padding, params::Base::Boolean(false))),
                1 => Ok((bytes + padding, params::Base::Boolean(true))),
                _ => Err(Error::InvalidBoolean),
            }
        }
        signature::Base::String => {
            let offset = offset + padding;
            let (bytes, string) = unmarshal_string(byteorder, &buf[offset..])?;
            Ok((bytes + padding, params::Base::String(string)))
        }
        signature::Base::ObjectPath => {
            // TODO validate
            let offset = offset + padding;
            let (bytes, string) = unmarshal_string(byteorder, &buf[offset..])?;
            Ok((bytes + padding, params::Base::ObjectPath(string)))
        }
        signature::Base::Signature => {
            // TODO validate
            let (bytes, string) = unmarshal_signature(buf)?;
            Ok((bytes, params::Base::Signature(string.to_owned())))
        }
    }
}
