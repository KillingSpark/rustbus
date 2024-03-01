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
    let param = match &sig {
        signature::Type::Base(base) => {
            let base = unmarshal_base(*base, ctx)?;
            params::Param::Base(base)
        }
        signature::Type::Container(cont) => {
            let cont = unmarshal_container(cont, ctx)?;
            params::Param::Container(cont)
        }
    };
    Ok(param)
}

pub fn unmarshal_variant(
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Variant<'static, 'static>> {
    let sig_str = ctx.read_signature()?;

    let mut sig = signature::Type::parse_description(sig_str)?;
    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(UnmarshalError::WrongSignature);
    }
    let sig = sig.remove(0);

    let param = unmarshal_with_sig(&sig, ctx)?;
    Ok(params::Variant { sig, value: param })
}

pub fn unmarshal_container(
    typ: &signature::Container,
    ctx: &mut UnmarshalContext,
) -> UnmarshalResult<params::Container<'static, 'static>> {
    let param = match typ {
        signature::Container::Array(elem_sig) => {
            let bytes_in_array = ctx.read_u32()? as usize;

            ctx.align_to(elem_sig.get_alignment())?;

            let mut elements = Vec::new();
            let mut ctx = ctx.sub_context(bytes_in_array)?;
            while !ctx.remainder().is_empty() {
                let element = unmarshal_with_sig(elem_sig, &mut ctx)?;
                elements.push(element);
            }

            params::Container::Array(params::Array {
                element_sig: elem_sig.as_ref().clone(),
                values: elements,
            })
        }
        signature::Container::Dict(key_sig, val_sig) => {
            let bytes_in_dict = ctx.read_u32()? as usize;

            ctx.align_to(8)?;

            let mut elements = std::collections::HashMap::new();
            let mut ctx = ctx.sub_context(bytes_in_dict)?;
            while !ctx.remainder().is_empty() {
                ctx.align_to(8)?;

                let key = unmarshal_base(*key_sig, &mut ctx)?;
                let val = unmarshal_with_sig(val_sig, &mut ctx)?;
                elements.insert(key, val);
            }

            params::Container::Dict(params::Dict {
                key_sig: *key_sig,
                value_sig: val_sig.as_ref().clone(),
                map: elements,
            })
        }
        signature::Container::Struct(sigs) => {
            ctx.align_to(8)?;
            let mut fields = Vec::new();

            if sigs.as_ref().is_empty() {
                return Err(UnmarshalError::EmptyStruct);
            }

            for field_sig in sigs.as_ref() {
                let field = unmarshal_with_sig(field_sig, ctx)?;
                fields.push(field);
            }
            params::Container::Struct(fields)
        }
        signature::Container::Variant => {
            let variant = unmarshal_variant(ctx)?;
            params::Container::Variant(Box::new(variant))
        }
    };
    Ok(param)
}
