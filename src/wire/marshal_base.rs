use crate::message;
use crate::wire::util::*;

pub fn marshal_base_param(
    byteorder: message::ByteOrder,
    p: &message::Base,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(p.sig().get_alignment(), buf);

    match p {
        message::Base::Boolean(b) => {
            if *b {
                write_u32(1, byteorder, buf);
            } else {
                write_u32(0, byteorder, buf);
            }
            Ok(())
        }
        message::Base::Byte(i) => {
            buf.push(*i);
            Ok(())
        }
        message::Base::Int16(i) => {
            write_u16(*i as u16, byteorder, buf);
            Ok(())
        }
        message::Base::Uint16(i) => {
            let raw = *i as u16;
            write_u16(raw, byteorder, buf);
            Ok(())
        }
        message::Base::Int32(i) => {
            write_u32(*i as u32, byteorder, buf);
            Ok(())
        }
        message::Base::Uint32(i) => {
            let raw = *i as u32;
            write_u32(raw, byteorder, buf);
            Ok(())
        }
        message::Base::UnixFd(i) => {
            let raw = *i as u32;
            write_u32(raw, byteorder, buf);
            Ok(())
        }
        message::Base::Int64(i) => {
            write_u64(*i as u64, byteorder, buf);
            Ok(())
        }
        message::Base::Uint64(i) => {
            let raw = *i as u64;
            write_u64(raw, byteorder, buf);
            Ok(())
        }
        message::Base::Double(i) => {
            let raw = *i as u64;
            write_u64(raw, byteorder, buf);
            Ok(())
        }
        message::Base::String(s) => {
            write_string(&s, byteorder, buf);
            Ok(())
        }
        message::Base::Signature(s) => {
            message::validate_signature(&s)?;
            write_signature(&s, buf);
            Ok(())
        }
        message::Base::ObjectPath(s) => {
            message::validate_object_path(&s)?;
            write_string(&s, byteorder, buf);
            Ok(())
        }
    }
}
