use crate::message;
use crate::params;

use crate::wire::marshal_container::marshal_param;
use crate::wire::util::*;

pub fn marshal(
    msg: &message::Message,
    byteorder: message::ByteOrder,
    header_fields: &[message::HeaderField],
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    marshal_header(msg, byteorder, header_fields, buf)?;
    pad_to_align(8, buf);
    let header_len = buf.len();

    for p in &msg.params {
        marshal_param(p, byteorder, buf)?;
    }

    // set the correct message length
    let body_len = buf.len() - header_len;
    insert_u32(byteorder, body_len as u32, &mut buf[4..8]);
    Ok(())
}

fn marshal_header(
    msg: &message::Message,
    byteorder: message::ByteOrder,
    header_fields: &[message::HeaderField],
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match byteorder {
        message::ByteOrder::BigEndian => {
            buf.push(b'B');
        }
        message::ByteOrder::LittleEndian => {
            buf.push(b'l');
        }
    }

    let msg_type = match msg.typ {
        message::MessageType::Invalid => return Err(message::Error::InvalidType),
        message::MessageType::Call => 1,
        message::MessageType::Reply => 2,
        message::MessageType::Error => 3,
        message::MessageType::Signal => 4,
    };
    buf.push(msg_type);

    // TODO Flags
    let flags = 0;
    buf.push(flags);

    // Version
    buf.push(1);

    // Zero bytes where the length of the message will be put
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);

    match msg.serial {
        Some(serial) => write_u32(serial, byteorder, buf),
        None => return Err(message::Error::NoSerial),
    }

    // Zero bytes where the length of the header fields will be put
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);

    if let Some(int) = &msg.interface {
        marshal_header_field(
            byteorder,
            &message::HeaderField::Interface(int.clone()),
            buf,
        )?;
    }
    if let Some(dest) = &msg.destination {
        marshal_header_field(
            byteorder,
            &message::HeaderField::Destination(dest.clone()),
            buf,
        )?;
    }
    if let Some(mem) = &msg.member {
        marshal_header_field(byteorder, &message::HeaderField::Member(mem.clone()), buf)?;
    }
    if let Some(obj) = &msg.object {
        marshal_header_field(byteorder, &message::HeaderField::Path(obj.clone()), buf)?;
    }
    if let Some(numfds) = &msg.num_fds {
        marshal_header_field(byteorder, &message::HeaderField::UnixFds(*numfds), buf)?;
    }
    if let Some(serial) = &msg.response_serial {
        marshal_header_field(byteorder, &message::HeaderField::ReplySerial(*serial), buf)?;
    }
    if !msg.params.is_empty() {
        let mut sig_str = String::new();
        for param in &msg.params {
            param.make_signature(&mut sig_str);
        }
        marshal_header_field(byteorder, &message::HeaderField::Signature(sig_str), buf)?;
    }
    marshal_header_fields(byteorder, header_fields, buf)?;
    let len = buf.len() - pos - 4; // -4 the bytes for the length indicator do not count
    insert_u32(byteorder, len as u32, &mut buf[pos..pos + 4]);

    Ok(())
}

fn marshal_header_field(
    byteorder: message::ByteOrder,
    field: &message::HeaderField,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    pad_to_align(8, buf);
    match field {
        message::HeaderField::Path(path) => {
            params::validate_object_path(path)?;
            buf.push(1);
            buf.push(1);
            buf.push(b'o');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&path, byteorder, buf);
        }
        message::HeaderField::Interface(int) => {
            params::validate_interface(int)?;
            buf.push(2);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&int, byteorder, buf);
        }
        message::HeaderField::Member(mem) => {
            params::validate_membername(mem)?;
            buf.push(3);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&mem, byteorder, buf);
        }
        message::HeaderField::ErrorName(name) => {
            params::validate_errorname(name)?;
            buf.push(4);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&name, byteorder, buf);
        }
        message::HeaderField::ReplySerial(rs) => {
            buf.push(5);
            buf.push(1);
            buf.push(b'u');
            buf.push(0);
            pad_to_align(4, buf);
            write_u32(*rs, byteorder, buf);
        }
        message::HeaderField::Destination(dest) => {
            params::validate_busname(dest)?;
            buf.push(6);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&dest, byteorder, buf);
        }
        message::HeaderField::Sender(snd) => {
            params::validate_busname(snd)?;
            buf.push(7);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&snd, byteorder, buf);
        }
        message::HeaderField::Signature(sig) => {
            params::validate_signature(sig)?;
            buf.push(8);
            buf.push(1);
            buf.push(b'g');
            buf.push(0);
            pad_to_align(4, buf);
            write_signature(&sig, buf);
        }
        message::HeaderField::UnixFds(fds) => {
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
    byteorder: message::ByteOrder,
    header_fields: &[message::HeaderField],
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    for field in header_fields {
        marshal_header_field(byteorder, field, buf)?;
    }
    Ok(())
}
