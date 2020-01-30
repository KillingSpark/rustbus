use crate::message;

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

fn pad_to_align(align_to: usize, buf: &mut Vec<u8>) {
    let padding_needed = align_to - (buf.len() % align_to);
    if padding_needed != align_to {
        buf.resize(buf.len() + padding_needed, 0);
        assert!(buf.len() % align_to == 0);
    }
}

pub fn write_u16(val: u16, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    insert_u16(byteorder, val, &mut buf[pos..]);
}
pub fn write_u32(val: u32, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    insert_u32(byteorder, val, &mut buf[pos..]);
}
pub fn write_u64(val: u64, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    insert_u64(byteorder, val, &mut buf[pos..]);
}

fn write_string(val: &str, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let len = val.len() as u32;
    write_u32(len, byteorder, buf);
    buf.extend(val.bytes());
    buf.push(0);
}

fn write_signature(val: &str, buf: &mut Vec<u8>) {
    let len = val.len() as u8;
    buf.push(len);
    buf.extend(val.bytes());
    buf.push(0);
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
            param.make_signature(&mut sig_str)?;
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
            message::validate_object_path(path)?;
            buf.push(1);
            buf.push(1);
            buf.push(b'o');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&path, byteorder, buf);
        }
        message::HeaderField::Interface(int) => {
            message::validate_interface(int)?;
            buf.push(2);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&int, byteorder, buf);
        }
        message::HeaderField::Member(mem) => {
            message::validate_membername(mem)?;
            buf.push(3);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&mem, byteorder, buf);
        }
        message::HeaderField::ErrorName(name) => {
            message::validate_errorname(name)?;
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
            message::validate_busname(dest)?;
            buf.push(6);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&dest, byteorder, buf);
        }
        message::HeaderField::Sender(snd) => {
            message::validate_busname(snd)?;
            buf.push(7);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(&snd, byteorder, buf);
        }
        message::HeaderField::Signature(sig) => {
            message::validate_signature(sig)?;
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

fn marshal_base_param(
    byteorder: message::ByteOrder,
    p: &message::Base,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        message::Base::Boolean(b) => {
            pad_to_align(4, buf);
            if *b {
                write_u32(0, byteorder, buf);
            } else {
                write_u32(1, byteorder, buf);
            }
            Ok(())
        }
        message::Base::Byte(i) => {
            buf.push(*i);
            Ok(())
        }
        message::Base::Int16(i) => {
            pad_to_align(2, buf);
            write_u16(*i as u16, byteorder, buf);
            Ok(())
        }
        message::Base::Uint16(i) => {
            let raw = *i as u16;
            pad_to_align(2, buf);
            write_u16(raw, byteorder, buf);
            Ok(())
        }
        message::Base::Int32(i) => {
            pad_to_align(4, buf);
            write_u32(*i as u32, byteorder, buf);
            Ok(())
        }
        message::Base::Uint32(i) => {
            let raw = *i as u32;
            pad_to_align(4, buf);
            write_u32(raw, byteorder, buf);
            Ok(())
        }
        message::Base::UnixFd(i) => {
            let raw = *i as u32;
            pad_to_align(4, buf);
            write_u32(raw, byteorder, buf);
            Ok(())
        }
        message::Base::Int64(i) => {
            pad_to_align(8, buf);
            write_u64(*i as u64, byteorder, buf);
            Ok(())
        }
        message::Base::Uint64(i) => {
            let raw = *i as u64;
            pad_to_align(8, buf);
            write_u64(raw, byteorder, buf);
            Ok(())
        }
        message::Base::Double(i) => {
            let raw = *i as u64;
            pad_to_align(8, buf);
            write_u64(raw, byteorder, buf);
            Ok(())
        }
        message::Base::String(s) => {
            pad_to_align(4, buf);
            write_string(&s, byteorder, buf);
            Ok(())
        }
        message::Base::Signature(s) => {
            message::validate_signature(&s)?;
            pad_to_align(1, buf);
            write_signature(&s, buf);
            Ok(())
        }
        message::Base::ObjectPath(s) => {
            message::validate_object_path(&s)?;
            pad_to_align(4, buf);
            write_string(&s, byteorder, buf);
            Ok(())
        }
    }
}

fn insert_u16(byteorder: message::ByteOrder, val: u16, buf: &mut [u8]) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
        }
        message::ByteOrder::BigEndian => {
            buf[0] = (val >> 8) as u8;
            buf[1] = (val) as u8;
        }
    }
}
fn insert_u32(byteorder: message::ByteOrder, val: u32, buf: &mut [u8]) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
            buf[2] = (val >> 16) as u8;
            buf[3] = (val >> 24) as u8;
        }
        message::ByteOrder::BigEndian => {
            buf[0] = (val >> 24) as u8;
            buf[1] = (val >> 16) as u8;
            buf[2] = (val >> 8) as u8;
            buf[3] = (val) as u8;
        }
    }
}
fn insert_u64(byteorder: message::ByteOrder, val: u64, buf: &mut [u8]) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
            buf[2] = (val >> 16) as u8;
            buf[3] = (val >> 24) as u8;
            buf[4] = (val >> 32) as u8;
            buf[5] = (val >> 40) as u8;
            buf[6] = (val >> 48) as u8;
            buf[7] = (val >> 56) as u8;
        }
        message::ByteOrder::BigEndian => {
            buf[7] = (val) as u8;
            buf[6] = (val >> 8) as u8;
            buf[5] = (val >> 16) as u8;
            buf[4] = (val >> 24) as u8;
            buf[3] = (val >> 32) as u8;
            buf[2] = (val >> 40) as u8;
            buf[1] = (val >> 48) as u8;
            buf[0] = (val >> 56) as u8;
        }
    }
}

fn marshal_container_param(
    p: &message::Container,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        message::Container::Array(params) => {
            message::validate_array(&params)?;
            pad_to_align(4, buf);
            let len_pos = buf.len();
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(0);

            // we need to pad here because the padding between length and first element does not count
            // into the length
            if let message::Param::Container(message::Container::Struct(_)) = params[0] {
                pad_to_align(8, buf);
            }
            let content_pos = buf.len();
            for p in params {
                marshal_param(&p, byteorder, buf)?;
            }
            let len = buf.len() - content_pos;
            insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
        }
        message::Container::Struct(params) => {
            pad_to_align(8, buf);
            for p in params {
                marshal_param(&p, byteorder, buf)?;
            }
        }
        message::Container::Dict(params) => {
            message::validate_dict(&params)?;
            pad_to_align(4, buf);
            let len_pos = buf.len();
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(0);
            pad_to_align(8, buf);

            let content_pos = buf.len();
            for (key, value) in params {
                pad_to_align(8, buf);
                marshal_base_param(byteorder, &key, buf)?;
                marshal_param(&value, byteorder, buf)?;
            }
            let len = buf.len() - content_pos;
            insert_u32(byteorder, len as u32, &mut buf[len_pos..len_pos + 4]);
        }
        message::Container::Variant(variant) => {
            let mut sig_str = String::new();
            variant.sig.to_str(&mut sig_str);
            buf.push(sig_str.len() as u8);
            buf.extend(sig_str.bytes());
            marshal_param(&variant.value, byteorder, buf)?;
        }
    }
    Ok(())
}

fn marshal_param(
    p: &message::Param,
    byteorder: message::ByteOrder,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    match p {
        message::Param::Base(b) => marshal_base_param(byteorder, &b, buf),
        message::Param::Container(c) => marshal_container_param(&c, byteorder, buf),
    }
}
