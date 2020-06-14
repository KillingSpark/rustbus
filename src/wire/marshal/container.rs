//! Marshal container params into raw bytes

use crate::params;
use crate::params::message;
use crate::signature;
use crate::wire::marshal::base::*;
use crate::wire::util::*;
use crate::ByteOrder;

pub fn marshal_param(
    p: &params::Param,
    byteorder: ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        params::Param::Base(b) => marshal_base_param(byteorder, &b, buf),
        params::Param::Container(c) => marshal_container_param(&c, byteorder, buf),
    }
}

fn marshal_array(
    array: &[params::Param],
    sig: &signature::Type,
    byteorder: ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(4, buf);
    let len_pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);

    // we need to pad here because the padding between length and first element does not count
    // into the length
    pad_to_align(sig.get_alignment(), buf);
    let content_pos = buf.len();
    for p in array {
        marshal_param(&p, byteorder, buf)?;
    }
    let len = buf.len() - content_pos;
    insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
    Ok(())
}

fn marshal_struct(
    params: &[params::Param],
    byteorder: ByteOrder,
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
    byteorder: ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    let mut sig_str = String::new();
    var.sig.to_str(&mut sig_str);
    marshal_signature(&sig_str, buf)?;
    marshal_param(&var.value, byteorder, buf)?;
    Ok(())
}

fn marshal_dict(
    dict: &params::DictMap,
    byteorder: ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(4, buf);
    let len_pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    pad_to_align(8, buf);

    let content_pos = buf.len();
    for (key, value) in dict {
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
    byteorder: ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        params::Container::Array(params) => {
            params::validate_array(&params.values, &params.element_sig)?;
            marshal_array(&params.values, &params.element_sig, byteorder, buf)?;
        }
        params::Container::ArrayRef(params) => {
            params::validate_array(&params.values, &params.element_sig)?;
            marshal_array(&params.values, &params.element_sig, byteorder, buf)?;
        }
        params::Container::Struct(params) => {
            marshal_struct(params, byteorder, buf)?;
        }
        params::Container::StructRef(params) => {
            marshal_struct(params, byteorder, buf)?;
        }
        params::Container::Dict(params) => {
            params::validate_dict(&params.map, params.key_sig, &params.value_sig)?;
            marshal_dict(&params.map, byteorder, buf)?;
        }
        params::Container::DictRef(params) => {
            params::validate_dict(&params.map, params.key_sig, &params.value_sig)?;
            marshal_dict(params.map, byteorder, buf)?;
        }
        params::Container::Variant(variant) => {
            marshal_variant(variant, byteorder, buf)?;
        }
    }
    Ok(())
}
