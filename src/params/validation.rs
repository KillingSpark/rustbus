use super::*;
use crate::message::*;
use crate::signature;

pub fn validate_object_path(op: &str) -> Result<()> {
    if op.is_empty() {
        return Err(Error::InvalidObjectPath);
    }
    if !op.starts_with('/') {
        return Err(Error::InvalidObjectPath);
    }
    if op.len() > 1 {
        let split = op.split('/').collect::<Vec<_>>();
        if split.len() < 2 {
            return Err(Error::InvalidObjectPath);
        }
        for element in &split[1..] {
            if element.is_empty() {
                return Err(Error::InvalidObjectPath);
            }
            if let Some(true) = element.chars().nth(0).map(|c| c.is_numeric()) {
                return Err(Error::InvalidObjectPath);
            }
            let alphanum_or_underscore = element.chars().all(|c| c.is_alphanumeric() || c == '_');
            if !alphanum_or_underscore {
                return Err(Error::InvalidObjectPath);
            }
        }
    }
    Ok(())
}
pub fn validate_interface(int: &str) -> Result<()> {
    if int.len() < 3 {
        return Err(Error::InvalidInterface);
    }
    if !int.contains('.') {
        return Err(Error::InvalidInterface);
    }

    let split = int.split('.').collect::<Vec<_>>();
    if split.len() < 2 {
        return Err(Error::InvalidInterface);
    }
    for element in split {
        if element.is_empty() {
            return Err(Error::InvalidInterface);
        }
        if let Some(true) = element.chars().nth(0).map(|c| c.is_numeric()) {
            return Err(Error::InvalidInterface);
        }
        let alphanum_or_underscore = element.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !alphanum_or_underscore {
            return Err(Error::InvalidInterface);
        }
    }

    Ok(())
}

pub fn validate_errorname(en: &str) -> Result<()> {
    validate_interface(en).map_err(|_| Error::InvalidErrorname)
}

pub fn validate_busname(bn: &str) -> Result<()> {
    if bn.len() < 3 {
        return Err(Error::InvalidBusname);
    }
    if !bn.contains('.') {
        return Err(Error::InvalidBusname);
    }

    let (unique, bn) = if bn.chars().nth(0).unwrap() == ':' {
        (true, &bn[1..])
    } else {
        (false, &bn[..])
    };

    let split = bn.split('.').collect::<Vec<_>>();
    if split.len() < 2 {
        return Err(Error::InvalidBusname);
    }

    for element in split {
        if element.is_empty() {
            return Err(Error::InvalidBusname);
        }
        if !unique && element.chars().nth(0).map(|c| c.is_numeric()) == Some(true) {
            return Err(Error::InvalidBusname);
        }
        let alphanum_or_underscore_or_dash = element
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
        if !alphanum_or_underscore_or_dash {
            return Err(Error::InvalidBusname);
        }
    }

    Ok(())
}

pub fn validate_membername(mem: &str) -> Result<()> {
    if mem.is_empty() {
        return Err(Error::InvalidMembername);
    }

    let alphanum_or_underscore = mem.chars().all(|c| c.is_alphanumeric() || c == '_');
    if !alphanum_or_underscore {
        return Err(Error::InvalidMembername);
    }

    Ok(())
}

pub fn validate_signature(sig: &str) -> Result<()> {
    signature::Type::parse_description(sig).map_err(Error::InvalidSignature)?;
    Ok(())
}

pub fn validate_array(array: &Array) -> Result<()> {
    // TODO check that all elements have the same type
    if array.values.is_empty() {
        return Ok(());
    }
    let mut first_sig = String::new();
    array.element_sig.to_str(&mut first_sig);
    let mut element_sig = String::new();
    for el in &array.values {
        element_sig.clear();
        el.make_signature(&mut element_sig);
        if !element_sig.eq(&first_sig) {
            return Err(Error::ArrayElementTypesDiffer);
        }
    }
    Ok(())
}

pub fn validate_dict(dict: &Dict) -> Result<()> {
    // TODO check that all elements have the same type
    if dict.map.is_empty() {
        return Ok(());
    }
    // check key sigs
    let mut first_sig = String::new();
    dict.key_sig.to_str(&mut first_sig);
    let mut element_sig = String::new();
    for el in dict.map.keys() {
        element_sig.clear();
        el.make_signature(&mut element_sig);
        if !element_sig.eq(&first_sig) {
            return Err(Error::DictKeyTypesDiffer);
        }
    }

    // check value sigs
    let mut first_sig = String::new();
    dict.value_sig.to_str(&mut first_sig);
    let mut element_sig = String::new();
    for el in dict.map.values() {
        element_sig.clear();
        el.make_signature(&mut element_sig);
        if !element_sig.eq(&first_sig) {
            return Err(Error::DictValueTypesDiffer);
        }
    }
    Ok(())
}

pub fn validate_header_fields(msg_type: MessageType, header_fields: &[HeaderField]) -> Result<()> {
    let mut have_path = false;
    let mut have_interface = false;
    let mut have_member = false;
    let mut have_errorname = false;
    let mut have_replyserial = false;
    let mut have_destination = false;
    let mut have_sender = false;
    let mut have_signature = false;
    let mut have_unixfds = false;

    for h in header_fields {
        match h {
            HeaderField::Destination(_) => {
                if have_destination {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_destination = true;
            }
            HeaderField::ErrorName(_) => {
                if have_errorname {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_errorname = true;
            }
            HeaderField::Interface(_) => {
                if have_interface {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_interface = true;
            }
            HeaderField::Member(_) => {
                if have_member {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_member = true;
            }
            HeaderField::Path(_) => {
                if have_path {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_path = true;
            }
            HeaderField::ReplySerial(_) => {
                if have_replyserial {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_replyserial = true;
            }
            HeaderField::Sender(_) => {
                if have_sender {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_sender = true;
            }
            HeaderField::Signature(_) => {
                if have_signature {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_signature = true;
            }
            HeaderField::UnixFds(_) => {
                if have_unixfds {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_unixfds = true;
            }
        }
    }

    let valid = match msg_type {
        MessageType::Invalid => false,
        MessageType::Call => have_path && have_member,
        MessageType::Signal => have_path && have_member && have_interface,
        MessageType::Reply => have_replyserial,
        MessageType::Error => have_errorname && have_replyserial,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidHeaderFields)
    }
}
