use crate::message;
use crate::params;
use crate::wire::marshal_base::*;
use crate::wire::util::*;

pub fn marshal_param(
    p: &params::Param,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        params::Param::Base(b) => marshal_base_param(byteorder, &b, buf),
        params::Param::Container(c) => marshal_container_param(&c, byteorder, buf),
    }
}

fn marshal_array(
    array: &params::Array,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    params::validate_array(array)?;
    pad_to_align(4, buf);
    let len_pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);

    // we need to pad here because the padding between length and first element does not count
    // into the length
    pad_to_align(array.element_sig.get_alignment(), buf);
    let content_pos = buf.len();
    for p in &array.values {
        marshal_param(&p, byteorder, buf)?;
    }
    let len = buf.len() - content_pos;
    insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
    Ok(())
}

fn marshal_struct(
    params: &Vec<params::Param>,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(8, buf);
    for p in params {
        marshal_param(&p, byteorder, buf)?;
    }
    Ok(())
}

fn marshal_variant(
    var: &params::Variant,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    let mut sig_str = String::new();
    var.sig.to_str(&mut sig_str);
    buf.push(sig_str.len() as u8);
    buf.extend(sig_str.bytes());
    marshal_param(&var.value, byteorder, buf)?;
    Ok(())
}

fn marshal_dict(
    dict: &params::Dict,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    params::validate_dict(&dict)?;
    pad_to_align(4, buf);
    let len_pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    pad_to_align(8, buf);

    let content_pos = buf.len();
    for (key, value) in &dict.map {
        pad_to_align(8, buf);
        marshal_base_param(byteorder, &key, buf)?;
        marshal_param(&value, byteorder, buf)?;
    }
    let len = buf.len() - content_pos;
    insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
    Ok(())
}

pub fn marshal_container_param(
    p: &params::Container,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        params::Container::Array(params) => {
            marshal_array(params, byteorder, buf)?;
        }
        params::Container::ArrayRef(params) => {
            marshal_array(*params, byteorder, buf)?;
        }
        params::Container::Struct(params) => {
            marshal_struct(params, byteorder, buf)?;
        }
        params::Container::StructRef(params) => {
            marshal_struct(params, byteorder, buf)?;
        }
        params::Container::Dict(params) => {
            marshal_dict(params, byteorder, buf)?;
        }
        params::Container::DictRef(params) => {
            marshal_dict(params, byteorder, buf)?;
        }
        params::Container::Variant(variant) => {
            marshal_variant(variant, byteorder, buf)?;
        }
        params::Container::VariantRef(variant) => {
            marshal_variant(variant, byteorder, buf)?;
        }
    }
    Ok(())
}
