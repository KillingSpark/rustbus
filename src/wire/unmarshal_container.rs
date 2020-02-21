use crate::message;
use crate::signature;
use crate::wire::unmarshal;
use crate::wire::unmarshal::UnmarshalResult;
use crate::wire::unmarshal_base::unmarshal_base;
use crate::wire::util::*;

pub fn unmarshal_with_sig(
    header: &unmarshal::Header,
    sig: &signature::Type,
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<message::Param> {
    let (bytes, param) = match &sig {
        signature::Type::Base(base) => {
            let (bytes, base) = unmarshal_base(header, buf, *base, offset)?;
            (bytes, message::Param::Base(base))
        }
        signature::Type::Container(cont) => {
            let (bytes, cont) = unmarshal_container(header, buf, cont, offset)?;
            (bytes, message::Param::Container(cont))
        }
    };
    Ok((bytes, param))
}

pub fn unmarshal_variant(
    header: &unmarshal::Header,
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<message::Variant> {
    let (sig_bytes_used, sig_str) = unmarshal_signature(&buf[offset..])?;
    let mut sig = signature::Type::parse_description(&sig_str)
        .map_err(|_| unmarshal::Error::InvalidSignature)?;
    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(unmarshal::Error::InvalidSignature);
    }
    let sig = sig.remove(0);
    let offset = offset + sig_bytes_used;

    let (param_bytes_used, param) = unmarshal_with_sig(header, &sig, buf, offset)?;
    Ok((
        sig_bytes_used + param_bytes_used,
        message::Variant { sig, value: param },
    ))
}

pub fn unmarshal_container(
    header: &unmarshal::Header,
    buf: &[u8],
    typ: &signature::Container,
    offset: usize,
) -> UnmarshalResult<message::Container> {
    let param = match typ {
        signature::Container::Array(elem_sig) => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let (_, bytes_in_array) = parse_u32(&buf[offset..], header.byteorder)?;
            let offset = offset + 4;

            let first_elem_padding = align_offset(elem_sig.get_alignment(), buf, offset)?;
            let offset = offset + first_elem_padding;

            let mut elements = Vec::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_array as usize {
                let (bytes_used, element) =
                    unmarshal_with_sig(header, &elem_sig, buf, offset + bytes_used_counter)?;
                elements.push(element);
                bytes_used_counter += bytes_used;
            }
            let total_bytes_used = padding + 4 + first_elem_padding + bytes_used_counter;

            (
                total_bytes_used,
                message::Container::Array(message::Array {
                    element_sig: elem_sig.as_ref().clone(),
                    values: elements,
                }),
            )
        }
        signature::Container::Dict(key_sig, val_sig) => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let (_, bytes_in_dict) = parse_u32(&buf[offset..], header.byteorder)?;
            let offset = offset + 4;

            let before_elements_padding = align_offset(8, buf, offset)?;
            let offset = offset + before_elements_padding;

            let mut elements = std::collections::HashMap::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_dict as usize {
                let element_padding = align_offset(8, buf, offset + bytes_used_counter)?;
                bytes_used_counter += element_padding;
                let (key_bytes, key) =
                    unmarshal_base(header, buf, *key_sig, offset + bytes_used_counter)?;
                bytes_used_counter += key_bytes;
                let (val_bytes, val) =
                    unmarshal_with_sig(header, val_sig, buf, offset + bytes_used_counter)?;
                bytes_used_counter += val_bytes;
                elements.insert(key, val);
            }
            (
                padding + before_elements_padding + 4 + bytes_used_counter,
                message::Container::Dict(message::Dict {
                    key_sig: *key_sig,
                    value_sig: val_sig.as_ref().clone(),
                    map: elements,
                }),
            )
        }
        signature::Container::Struct(sigs) => {
            let padding = align_offset(8, buf, offset)?;
            let offset = offset + padding;
            let mut fields = Vec::new();

            let mut bytes_used_counter = 0;
            for field_sig in sigs {
                let (bytes_used, field) =
                    unmarshal_with_sig(header, field_sig, buf, offset + bytes_used_counter)?;
                fields.push(field);
                bytes_used_counter += bytes_used;
            }
            (
                padding + bytes_used_counter,
                message::Container::Struct(fields),
            )
        }
        signature::Container::Variant => {
            let (bytes_used, variant) = unmarshal_variant(header, buf, offset)?;
            (bytes_used, message::Container::Variant(Box::new(variant)))
        }
    };
    Ok(param)
}
