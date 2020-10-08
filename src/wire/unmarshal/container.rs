//! Unmarshal container params from raw bytes

use crate::params;
use crate::signature;
use crate::wire::unmarshal;
use crate::wire::unmarshal::base::unmarshal_base;
use crate::wire::unmarshal::UnmarshalContext;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::util::*;

pub fn unmarshal_with_sig<'a, 'e>(
    sig: &signature::Type,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Param<'a, 'e>> {
    let (bytes, param) = match &sig {
        signature::Type::Base(base) => {
            let (bytes, base) = unmarshal_base(*base, ctx)?;
            (bytes, params::Param::Base(base))
        }
        signature::Type::Container(cont) => {
            let (bytes, cont) = unmarshal_container(cont, ctx)?;
            (bytes, params::Param::Container(cont))
        }
    };
    Ok((bytes, param))
}

pub fn unmarshal_variant<'a, 'e>(
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Variant<'a, 'e>> {
    let (sig_bytes_used, sig_str) = unmarshal_signature(&ctx.buf[ctx.offset..])?;

    let mut sig = signature::Type::parse_description(&sig_str)?;
    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(unmarshal::Error::WrongSignature);
    }
    let sig = sig.remove(0);
    ctx.offset += sig_bytes_used;

    let (param_bytes_used, param) = unmarshal_with_sig(&sig, ctx)?;
    Ok((
        sig_bytes_used + param_bytes_used,
        params::Variant { sig, value: param },
    ))
}

pub fn unmarshal_container<'a, 'e>(
    typ: &signature::Container,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Container<'a, 'e>> {
    let param = match typ {
        signature::Container::Array(elem_sig) => {
            let padding = align_offset(4, ctx.buf, ctx.offset)?;
            ctx.offset += padding;

            let (_, bytes_in_array) = parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)?;
            ctx.offset += 4;

            let first_elem_padding = align_offset(elem_sig.get_alignment(), ctx.buf, ctx.offset)?;
            ctx.offset += first_elem_padding;

            let mut elements = Vec::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_array as usize {
                let (bytes_used, element) = unmarshal_with_sig(&elem_sig, ctx)?;
                elements.push(element);
                bytes_used_counter += bytes_used;
            }
            let total_bytes_used = padding + 4 + first_elem_padding + bytes_used_counter;

            (
                total_bytes_used,
                params::Container::Array(params::Array {
                    element_sig: elem_sig.as_ref().clone(),
                    values: elements,
                }),
            )
        }
        signature::Container::Dict(key_sig, val_sig) => {
            let padding = align_offset(4, ctx.buf, ctx.offset)?;
            ctx.offset += padding;
            let (_, bytes_in_dict) = parse_u32(&ctx.buf[ctx.offset..], ctx.byteorder)?;
            ctx.offset += 4;

            let before_elements_padding = align_offset(8, ctx.buf, ctx.offset)?;
            ctx.offset += before_elements_padding;

            let mut elements = std::collections::HashMap::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_dict as usize {
                let element_padding = align_offset(8, ctx.buf, ctx.offset)?;
                bytes_used_counter += element_padding;
                ctx.offset += element_padding;

                let (key_bytes, key) = unmarshal_base(*key_sig, ctx)?;
                bytes_used_counter += key_bytes;

                let (val_bytes, val) = unmarshal_with_sig(val_sig, ctx)?;
                bytes_used_counter += val_bytes;

                elements.insert(key, val);
            }
            (
                padding + before_elements_padding + 4 + bytes_used_counter,
                params::Container::Dict(params::Dict {
                    key_sig: *key_sig,
                    value_sig: val_sig.as_ref().clone(),
                    map: elements,
                }),
            )
        }
        signature::Container::Struct(sigs) => {
            let padding = align_offset(8, ctx.buf, ctx.offset)?;
            ctx.offset += padding;
            let mut fields = Vec::new();

            let mut bytes_used_counter = 0;
            for field_sig in sigs {
                let (bytes_used, field) = unmarshal_with_sig(field_sig, ctx)?;
                fields.push(field);
                bytes_used_counter += bytes_used;
            }
            (
                padding + bytes_used_counter,
                params::Container::Struct(fields),
            )
        }
        signature::Container::Variant => {
            let (bytes_used, variant) = unmarshal_variant(ctx)?;
            (bytes_used, params::Container::Variant(Box::new(variant)))
        }
    };
    Ok(param)
}
