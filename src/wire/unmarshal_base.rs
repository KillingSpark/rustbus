use crate::message;
use crate::signature;
use crate::wire::util::*;
use crate::wire::unmarshal::Header;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::unmarshal::Error;

pub fn unmarshal_base(
    header: &Header,
    buf: &[u8],
    typ: signature::Base,
    offset: usize,
) -> UnmarshalResult<message::Base> {
    let padding = align_offset(typ.get_alignment(), buf, offset)?;

    match typ {
        signature::Base::Byte => {
            if buf.is_empty() {
                return Err(Error::NotEnoughBytes);
            }
            Ok((1, message::Base::Byte(buf[offset])))
        }
        signature::Base::Uint16 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 2];
            let (bytes, val) = parse_u16(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Uint16(val)))
        }
        signature::Base::Int16 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 2];
            let (bytes, val) = parse_u16(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Int16(val as i16)))
        }
        signature::Base::Uint32 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Uint32(val)))
        }
        signature::Base::UnixFd => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::UnixFd(val)))
        }
        signature::Base::Int32 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Int32(val as i32)))
        }
        signature::Base::Uint64 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Uint64(val)))
        }
        signature::Base::Int64 => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Int64(val as i64)))
        }
        signature::Base::Double => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Double(val)))
        }
        signature::Base::Boolean => {
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            match val {
                0 => Ok((bytes + padding, message::Base::Boolean(false))),
                1 => Ok((bytes + padding, message::Base::Boolean(true))),
                _ => Err(Error::InvalidBoolean),
            }
        }
        signature::Base::String => {
            let offset = offset + padding;
            let (bytes, string) = unmarshal_string(header, &buf[offset..])?;
            Ok((bytes + padding, message::Base::String(string)))
        }
        signature::Base::ObjectPath => {
            // TODO validate
            let offset = offset + padding;
            let (bytes, string) = unmarshal_string(header, &buf[offset..])?;
            Ok((bytes + padding, message::Base::ObjectPath(string)))
        }
        signature::Base::Signature => {
            // TODO validate
            let (bytes, string) = unmarshal_signature(buf)?;
            Ok((bytes, message::Base::Signature(string)))
        }
    }
}