use crate::message;
use crate::params;
use crate::wire::util::*;

pub fn marshal_base_param(
    byteorder: message::ByteOrder,
    p: &params::Base,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(p.sig().get_alignment(), buf);

    match p {
        params::Base::Boolean(b) => {
            if *b {
                write_u32(1, byteorder, buf);
            } else {
                write_u32(0, byteorder, buf);
            }
            Ok(())
        }
        params::Base::Byte(i) => {
            buf.push(*i);
            Ok(())
        }
        params::Base::Int16(i) => {
            write_u16(*i as u16, byteorder, buf);
            Ok(())
        }
        params::Base::Uint16(i) => {
            let raw = *i as u16;
            write_u16(raw, byteorder, buf);
            Ok(())
        }
        params::Base::Int32(i) => {
            write_u32(*i as u32, byteorder, buf);
            Ok(())
        }
        params::Base::Uint32(i) => {
            let raw = *i as u32;
            write_u32(raw, byteorder, buf);
            Ok(())
        }
        params::Base::UnixFd(i) => {
            let raw = *i as u32;
            write_u32(raw, byteorder, buf);
            Ok(())
        }
        params::Base::Int64(i) => {
            write_u64(*i as u64, byteorder, buf);
            Ok(())
        }
        params::Base::Uint64(i) => {
            let raw = *i as u64;
            write_u64(raw, byteorder, buf);
            Ok(())
        }
        params::Base::Double(i) => {
            let raw = *i as u64;
            write_u64(raw, byteorder, buf);
            Ok(())
        }
        params::Base::String(s) => {
            write_string(&s, byteorder, buf);
            Ok(())
        }
        params::Base::Signature(s) => {
            params::validate_signature(&s)?;
            write_signature(&s, buf);
            Ok(())
        }
        params::Base::ObjectPath(s) => {
            params::validate_object_path(&s)?;
            write_string(&s, byteorder, buf);
            Ok(())
        }
    }
}
