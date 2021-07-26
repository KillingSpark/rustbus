//! Various validation functions for e.g. ObjectPath constraints

use super::*;
use crate::message_builder::MessageType;
use crate::params;
use crate::signature;
use crate::wire::HeaderField;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidSignature(signature::Error),
    InvalidObjectPath,
    InvalidBusname,
    InvalidErrorname,
    InvalidMembername,
    InvalidInterface,
    InvalidHeaderFields,
    StringContainsNullByte,
    InvalidUtf8,
    DuplicatedHeaderFields,
    ArrayElementTypesDiffer,
    DictKeyTypesDiffer,
    DictValueTypesDiffer,
}

type Result<T> = std::result::Result<T, Error>;

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
            let alphanum_or_underscore = element.chars().all(|c| c.is_alphanumeric() || c == '_');
            if !alphanum_or_underscore {
                return Err(Error::InvalidObjectPath);
            }
        }
    }
    Ok(())
}
pub fn validate_interface(int: &str) -> Result<()> {
    let split = int.split('.');
    let mut cnt = 0;
    for (i, element) in split.enumerate() {
        if element
            .chars()
            .next()
            .ok_or(Error::InvalidInterface)?
            .is_numeric()
        {
            return Err(Error::InvalidInterface);
        }
        let alphanum_or_underscore = element.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !alphanum_or_underscore {
            return Err(Error::InvalidInterface);
        }
        cnt = i + 1;
    }
    if cnt >= 2 {
        Ok(())
    } else {
        Err(Error::InvalidInterface)
    }
}

#[inline]
pub fn validate_errorname(en: &str) -> Result<()> {
    validate_interface(en).map_err(|_| Error::InvalidErrorname)
}

pub fn validate_busname(bn: &str) -> Result<()> {
    let (unique, bus_name) = if let Some(unique_name) = bn.strip_prefix(':') {
        (true, unique_name)
    } else {
        (false, bn)
    };

    let split = bus_name.split('.');
    let mut cnt = 0;
    for (i, element) in split.enumerate() {
        if element
            .chars()
            .next()
            .ok_or(Error::InvalidBusname)?
            .is_numeric()
            && !unique
        {
            return Err(Error::InvalidBusname);
        }
        let alphanum_or_underscore_or_dash = element
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
        if !alphanum_or_underscore_or_dash {
            return Err(Error::InvalidBusname);
        }
        cnt = i + 1;
    }
    if cnt >= 2 {
        Ok(())
    } else {
        Err(Error::InvalidBusname)
    }
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
    const MAX_BRACKET_DEPTH: usize = 32;

    if sig.len() > 255 {
        return Err(Error::InvalidSignature(signature::Error::SignatureTooLong));
    }

    let mut pos = 0;
    while pos < sig.len() {
        pos += validate_next(sig.as_bytes(), pos, 0, 0)?;
    }

    // recursive function to validate the next signature after pos
    fn validate_next(
        sig: &[u8],
        pos: usize,
        array_depth: usize,
        bracket_depth: usize,
    ) -> Result<usize> {
        if bracket_depth > MAX_BRACKET_DEPTH {
            return Err(Error::InvalidSignature(signature::Error::NestingTooDeep));
        }
        if array_depth > MAX_BRACKET_DEPTH {
            return Err(Error::InvalidSignature(signature::Error::NestingTooDeep));
        }
        if pos >= sig.len() {
            return Err(Error::InvalidSignature(signature::Error::InvalidSignature));
        }

        match sig[pos] {
            b'y' | b'b' | b'n' | b'q' | b'i' | b'u' | b'x' | b't' | b'd' | b'h' | b's' | b'o'
            | b'g' | b'v' => {
                // Nothing to do just skip base types and variants
                Ok(1)
            }
            b'a' => {
                let element_sig_len = validate_next(sig, pos + 1, array_depth + 1, bracket_depth)?;
                Ok(element_sig_len + 1)
            }
            b'{' => {
                if pos > 0 && sig.len() > pos + 2 && sig[pos - 1] == b'a' {
                    match sig[pos + 1] {
                        b'y' | b'b' | b'n' | b'q' | b'i' | b'u' | b'x' | b't' | b'd' | b'h'
                        | b's' | b'o' | b'g' => {
                            // Nothing to do just skip base types
                        }
                        _ => {
                            return Err(Error::InvalidSignature(signature::Error::InvalidSignature))
                        }
                    }
                    let val_sig_len = validate_next(sig, pos + 2, array_depth, bracket_depth + 1)?;
                    let inner_sigs_len = 1 + val_sig_len;
                    if pos + inner_sigs_len + 1 >= sig.len() {
                        Err(Error::InvalidSignature(signature::Error::InvalidSignature))
                    } else if sig[pos + inner_sigs_len + 1] == b'}' {
                        Ok(inner_sigs_len + 2)
                    } else {
                        Err(Error::InvalidSignature(signature::Error::InvalidSignature))
                    }
                } else {
                    // there must be an 'a' before the '{'
                    Err(Error::InvalidSignature(signature::Error::InvalidSignature))
                }
            }
            b'(' => {
                let mut counter = 1;
                loop {
                    if pos + counter >= sig.len() {
                        return Err(Error::InvalidSignature(signature::Error::InvalidSignature));
                    }
                    if sig[pos + counter] == b')' {
                        counter += 1;
                        break;
                    }
                    let elem_sig_len =
                        validate_next(sig, pos + counter, array_depth, bracket_depth + 1)?;
                    counter += elem_sig_len;
                }
                Ok(counter)
            }
            _ => Err(Error::InvalidSignature(signature::Error::InvalidSignature)),
        }
    }

    Ok(())
}

pub fn validate_array<'a, 'e>(array: &[Param<'a, 'e>], sig: &signature::Type) -> Result<()> {
    if array.is_empty() {
        return Ok(());
    }
    for el in array {
        if !sig.eq(&el.sig()) {
            return Err(Error::ArrayElementTypesDiffer);
        }
    }
    Ok(())
}

pub fn validate_dict(
    dict: &params::DictMap,
    key_sig: signature::Base,
    val_sig: &signature::Type,
) -> Result<()> {
    if dict.is_empty() {
        return Ok(());
    }
    let key_sig = signature::Type::Base(key_sig);
    for el in dict.keys() {
        if !key_sig.eq(&el.sig()) {
            return Err(Error::DictKeyTypesDiffer);
        }
    }

    for el in dict.values() {
        if !val_sig.eq(&el.sig()) {
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

// more specific tests for constraints on strings
#[test]
fn test_objectpath_constraints() {
    let no_beginning_slash = "da/di/du";
    assert_eq!(
        Err(Error::InvalidObjectPath),
        crate::params::validate_object_path(no_beginning_slash)
    );
    let empty_element = "/da//du";
    assert_eq!(
        Err(Error::InvalidObjectPath),
        crate::params::validate_object_path(empty_element)
    );
    let trailing_slash = "/da/di/du/";
    assert_eq!(
        Err(Error::InvalidObjectPath),
        crate::params::validate_object_path(trailing_slash)
    );
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(Error::InvalidObjectPath),
        crate::params::validate_object_path(invalid_chars)
    );
    let trailing_slash_on_root = "/";
    assert_eq!(
        Ok(()),
        crate::params::validate_object_path(trailing_slash_on_root)
    );
}
#[test]
fn test_interface_constraints() {
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(Error::InvalidInterface),
        crate::params::validate_interface(invalid_chars)
    );
    let leading_digits = "1leading.digits";
    assert_eq!(
        Err(Error::InvalidInterface),
        crate::params::validate_interface(leading_digits)
    );
    let too_short = "have_more_than_one_element";
    assert_eq!(
        Err(Error::InvalidInterface),
        crate::params::validate_interface(too_short)
    );
    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s.push('.');
        s
    });
    assert_eq!(
        Err(Error::InvalidInterface),
        crate::params::validate_interface(&too_long)
    );
}
#[test]
fn test_busname_constraints() {
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(Error::InvalidBusname),
        crate::params::validate_busname(invalid_chars)
    );
    let empty = "";
    assert_eq!(
        Err(Error::InvalidBusname),
        crate::params::validate_busname(empty)
    );
    let too_short = "have_more_than_one_element";
    assert_eq!(
        Err(Error::InvalidBusname),
        crate::params::validate_busname(too_short)
    );

    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s.push('.');
        s
    });
    assert_eq!(
        Err(Error::InvalidBusname),
        crate::params::validate_busname(&too_long)
    );
}
#[test]
fn test_membername_constraints() {
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(Error::InvalidMembername),
        crate::params::validate_membername(invalid_chars)
    );
    let dots = "Shouldnt.have.dots";
    assert_eq!(
        Err(Error::InvalidMembername),
        crate::params::validate_membername(dots)
    );
    let empty = "";
    assert_eq!(
        Err(Error::InvalidMembername),
        crate::params::validate_membername(empty)
    );

    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s.push('.');
        s
    });
    assert_eq!(
        Err(Error::InvalidMembername),
        crate::params::validate_membername(&too_long)
    );
}
#[test]
fn test_signature_constraints() {
    let wrong_parans = "((i)";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let wrong_parans = "(i))";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let wrong_parans = "a{{i}";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let wrong_parans = "a{i}}";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let array_without_type = "(i)a";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(array_without_type)
    );
    let invalid_chars = "!!§$%&(i)a";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(invalid_chars)
    );

    let too_deep_nesting = "(((((((((((((((((((((((((((((((((y)))))))))))))))))))))))))))))))))";
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::NestingTooDeep
        )),
        crate::params::validate_signature(too_deep_nesting)
    );

    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s
    });
    assert_eq!(
        Err(Error::InvalidSignature(
            crate::signature::Error::SignatureTooLong
        )),
        crate::params::validate_signature(&too_long)
    );
}
