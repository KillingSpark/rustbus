use crate::message;
use crate::wire::util::*;
use crate::wire::marshal_base::*;

pub fn marshal_param(
    p: &message::Param,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        message::Param::Base(b) => marshal_base_param(byteorder, &b, buf),
        message::Param::Container(c) => marshal_container_param(&c, byteorder, buf),
    }
}

pub fn marshal_container_param(
    p: &message::Container,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        message::Container::Array(params) => {
            message::validate_array(&params)?;
            pad_to_align(4, buf);
            let len_pos = buf.len();
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(0);

            // we need to pad here because the padding between length and first element does not count
            // into the length
            pad_to_align(params.element_sig.get_alignment(), buf);
            let content_pos = buf.len();
            for p in &params.values {
                marshal_param(&p, byteorder, buf)?;
            }
            let len = buf.len() - content_pos;
            insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
        }
        message::Container::Struct(params) => {
            pad_to_align(8, buf);
            for p in params {
                marshal_param(&p, byteorder, buf)?;
            }
        }
        message::Container::Dict(params) => {
            message::validate_dict(&params)?;
            pad_to_align(4, buf);
            let len_pos = buf.len();
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(0);
            pad_to_align(8, buf);

            let content_pos = buf.len();
            for (key, value) in &params.map {
                pad_to_align(8, buf);
                marshal_base_param(byteorder, &key, buf)?;
                marshal_param(&value, byteorder, buf)?;
            }
            let len = buf.len() - content_pos;
            insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
        }
        message::Container::Variant(variant) => {
            let mut sig_str = String::new();
            variant.sig.to_str(&mut sig_str);
            buf.push(sig_str.len() as u8);
            buf.extend(sig_str.bytes());
            marshal_param(&variant.value, byteorder, buf)?;
        }
    }
    Ok(())
}