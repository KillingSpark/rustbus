//! Unmarshal container params from raw bytes

use crate::params;
use crate::signature;
use crate::wire::errors::UnmarshalError;
use crate::wire::unmarshal::base::unmarshal_base;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::unmarshal_context::UnmarshalContext;

pub fn unmarshal_with_sig(
    sig: &signature::Type,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Param<'static, 'static>> {
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

pub fn unmarshal_variant(
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Variant<'static, 'static>> {
    let (sig_bytes_used, sig_str) = ctx.read_signature()?;

    let mut sig = signature::Type::parse_description(sig_str)?;
    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(UnmarshalError::WrongSignature);
    }
    let sig = sig.remove(0);

    let (param_bytes_used, param) = unmarshal_with_sig(&sig, ctx)?;
    Ok((
        sig_bytes_used + param_bytes_used,
        params::Variant { sig, value: param },
    ))
}

pub fn unmarshal_container(
    typ: &signature::Container,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Container<'static, 'static>> {
    let param = match typ {
        signature::Container::Array(elem_sig) => {
            let start_len = ctx.remainder().len();

            let (bytes_in_len, bytes_in_array) = ctx.read_u32()?;

            ctx.align_to(elem_sig.get_alignment())?;

            let mut elements = Vec::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_array as usize {
                let (bytes_used, element) = unmarshal_with_sig(elem_sig, ctx)?;
                elements.push(element);
                bytes_used_counter += bytes_used;
            }

            let total_bytes_used = start_len - ctx.remainder().len();
            debug_assert_eq!(total_bytes_used, bytes_used_counter + bytes_in_len);
            (
                total_bytes_used,
                params::Container::Array(params::Array {
                    element_sig: elem_sig.as_ref().clone(),
                    values: elements,
                }),
            )
        }
        signature::Container::Dict(key_sig, val_sig) => {
            let start_len = ctx.remainder().len();

            let (bytes_in_len, bytes_in_dict) = ctx.read_u32()?;

            ctx.align_to(8)?;

            let mut elements = std::collections::HashMap::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_dict as usize {
                bytes_used_counter += ctx.align_to(8)?;

                let (key_bytes, key) = unmarshal_base(*key_sig, ctx)?;
                bytes_used_counter += key_bytes;

                let (val_bytes, val) = unmarshal_with_sig(val_sig, ctx)?;
                bytes_used_counter += val_bytes;

                elements.insert(key, val);
            }

            let total_bytes_used = start_len - ctx.remainder().len();
            debug_assert_eq!(total_bytes_used, bytes_used_counter + bytes_in_len);
            (
                total_bytes_used,
                params::Container::Dict(params::Dict {
                    key_sig: *key_sig,
                    value_sig: val_sig.as_ref().clone(),
                    map: elements,
                }),
            )
        }
        signature::Container::Struct(sigs) => {
            let start_len = ctx.remainder().len();

            ctx.align_to(8)?;
            let mut fields = Vec::new();

            if sigs.as_ref().is_empty() {
                return Err(UnmarshalError::EmptyStruct);
            }

            for field_sig in sigs.as_ref() {
                let (_, field) = unmarshal_with_sig(field_sig, ctx)?;
                fields.push(field);
            }
            let total_bytes_used = start_len - ctx.remainder().len();
            (total_bytes_used, params::Container::Struct(fields))
        }
        signature::Container::Variant => {
            let (bytes_used, variant) = unmarshal_variant(ctx)?;
            (bytes_used, params::Container::Variant(Box::new(variant)))
        }
    };
    Ok(param)
}
