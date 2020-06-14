//! All things relevant to marshalling content into raw bytes
//!
//! * `base` and `container` are for the Param approach that map dbus concepts to enums/structs
//! * `traits` is for the trait based approach

use crate::message_builder;
use crate::params;
use crate::params::message;
use crate::wire::HeaderField;
use crate::ByteOrder;

use crate::wire::util::*;

pub mod base;
pub mod container;
pub mod traits;

pub fn marshal(
    msg: &crate::message_builder::MarshalledMessage,
    byteorder: ByteOrder,
    header_fields: &[HeaderField],
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    marshal_header(msg, byteorder, header_fields, buf)?;
    pad_to_align(8, buf);
    let header_len = buf.len();

    buf.extend_from_slice(msg.get_buf());

    // set the correct message length
    let body_len = buf.len() - header_len;
    insert_u32(byteorder, body_len as u32, &mut buf[4..8]);
    Ok(())
}

fn marshal_header(
    msg: &crate::message_builder::MarshalledMessage,
    byteorder: ByteOrder,
    header_fields: &[HeaderField],
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match byteorder {
        ByteOrder::BigEndian => {
            buf.push(b'B');
        }
        ByteOrder::LittleEndian => {
            buf.push(b'l');
        }
    }

    let msg_type = match msg.typ {
        message_builder::MessageType::Invalid => return Err(crate::Error::InvalidType),
        message_builder::MessageType::Call => 1,
        message_builder::MessageType::Reply => 2,
        message_builder::MessageType::Error => 3,
        message_builder::MessageType::Signal => 4,
    };
    buf.push(msg_type);

    buf.push(msg.flags);

    // Version
    buf.push(1);

    // Zero bytes where the length of the message will be put
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);

    match msg.dynheader.serial {
        Some(serial) => write_u32(serial, byteorder, buf),
        None => return Err(crate::wire::unmarshal::Error::NoSerial.into()),
    }

    // Zero bytes where the length of the header fields will be put
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);

    if let Some(int) = &msg.dynheader.interface {
        marshal_header_field(byteorder, &HeaderField::Interface(int.clone()), buf)?;
    }
    if let Some(dest) = &msg.dynheader.destination {
        marshal_header_field(byteorder, &HeaderField::Destination(dest.clone()), buf)?;
    }
    if let Some(mem) = &msg.dynheader.member {
        marshal_header_field(byteorder, &HeaderField::Member(mem.clone()), buf)?;
    }
    if let Some(obj) = &msg.dynheader.object {
        marshal_header_field(byteorder, &HeaderField::Path(obj.clone()), buf)?;
    }
    if let Some(numfds) = &msg.dynheader.num_fds {
        marshal_header_field(byteorder, &HeaderField::UnixFds(*numfds), buf)?;
    }
    if let Some(serial) = &msg.dynheader.response_serial {
        marshal_header_field(byteorder, &HeaderField::ReplySerial(*serial), buf)?;
    }
    if !msg.get_buf().is_empty() {
        let sig_str = msg.get_sig().to_owned();
        marshal_header_field(byteorder, &HeaderField::Signature(sig_str), buf)?;
    }
    marshal_header_fields(byteorder, header_fields, buf)?;
    let len = buf.len() - pos - 4; // -4 the bytes for the length indicator do not count
    insert_u32(byteorder, len as u32, &mut buf[pos..pos + 4]);

    Ok(())
}

fn marshal_header_field(
    byteorder: ByteOrder,
    field: &HeaderField,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(8, buf);
    match field {
        HeaderField::Path(path) => {
            params::validate_object_path(path)?;
            buf.push(1);
            buf.push(1);
            buf.push(b'o');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&path, byteorder, buf);
        }
        HeaderField::Interface(int) => {
            params::validate_interface(int)?;
            buf.push(2);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&int, byteorder, buf);
        }
        HeaderField::Member(mem) => {
            params::validate_membername(mem)?;
            buf.push(3);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&mem, byteorder, buf);
        }
        HeaderField::ErrorName(name) => {
            params::validate_errorname(name)?;
            buf.push(4);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&name, byteorder, buf);
        }
        HeaderField::ReplySerial(rs) => {
            buf.push(5);
            buf.push(1);
            buf.push(b'u');
            buf.push(0);
            pad_to_align(4, buf);
            write_u32(*rs, byteorder, buf);
        }
        HeaderField::Destination(dest) => {
            params::validate_busname(dest)?;
            buf.push(6);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&dest, byteorder, buf);
        }
        HeaderField::Sender(snd) => {
            params::validate_busname(snd)?;
            buf.push(7);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&snd, byteorder, buf);
        }
        HeaderField::Signature(sig) => {
            params::validate_signature(sig)?;
            buf.push(8);
            buf.push(1);
            buf.push(b'g');
            buf.push(0);
            pad_to_align(4, buf);
            write_signature(&sig, buf);
        }
        HeaderField::UnixFds(fds) => {
            buf.push(9);
            buf.push(1);
            buf.push(b'u');
            buf.push(0);
            pad_to_align(4, buf);
            write_u32(*fds, byteorder, buf);
        }
    }
    Ok(())
}

fn marshal_header_fields(
    byteorder: ByteOrder,
    header_fields: &[HeaderField],
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    for field in header_fields {
        marshal_header_field(byteorder, field, buf)?;
    }
    Ok(())
}
