use crate::message;

pub fn marshal(
    msg: &message::Message,
    byteorder: message::ByteOrder,
    serial: u32,
    header_fields: &Vec<message::HeaderField>,
    buf: &mut Vec<u8>,
) -> message::Result<()> {
    marshal_header(msg, byteorder, serial, header_fields, buf)?;
    let header_len = buf.len();

    // TODO marshal interface and member
    for p in &msg.params {
        marshal_param(p, buf)?;
    }

    // set the correct message length
    let body_len = buf.len() - header_len;
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[4] = (body_len >> 0) as u8;
            buf[5] = (body_len >> 8) as u8;
            buf[6] = (body_len >> 16) as u8;
            buf[7] = (body_len >> 24) as u8;
        }
        message::ByteOrder::BigEndian => {
            buf[4] = (body_len >> 24) as u8;
            buf[5] = (body_len >> 16) as u8;
            buf[6] = (body_len >> 8) as u8;
            buf[7] = (body_len >> 0) as u8;
        }
    }
    Ok(())
}

fn pad_to_align(align_to: usize, buf: &mut Vec<u8>) {
    let padding_needed = buf.len() % align_to;
    buf.resize(buf.len() + padding_needed, 0);
}

fn write_u32(val: u32, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf.push((val >> 0) as u8);
            buf.push((val >> 8) as u8);
            buf.push((val >> 16) as u8);
            buf.push((val >> 24) as u8);
        }
        message::ByteOrder::BigEndian => {
            buf.push((val >> 24) as u8);
            buf.push((val >> 16) as u8);
            buf.push((val >> 8) as u8);
            buf.push((val >> 0) as u8);
        }
    }
}

fn write_string(val: &str, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let len = val.len() as u32;
    write_u32(len, byteorder, buf);
    buf.push((len >> 0) as u8);
    buf.push((len >> 8) as u8);
    buf.push((len >> 16) as u8);
    buf.push((len >> 24) as u8);
    buf.extend(val.bytes());
    buf.push(0);
}

fn marshal_header(
    msg: &message::Message,
    byteorder: message::ByteOrder,
    serial: u32,
    header_fields: &Vec<message::HeaderField>,
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

    write_u32(serial, byteorder, buf);

    marshal_header_fields(byteorder, header_fields, buf);

    pad_to_align(8, buf);
    Ok(())
}

fn marshal_header_fields(
    byteorder: message::ByteOrder,
    header_fields: &Vec<message::HeaderField>,
    buf: &mut Vec<u8>,
) {
    for field in header_fields {
        match field {
            message::HeaderField::Path(path) => {
                buf.push(1);
                buf.push(1);
                buf.push(b'o');
                pad_to_align(4, buf);
                write_string(&path, byteorder, buf);
            }
            message::HeaderField::Interface(int) => {
                buf.push(2);
                buf.push(1);
                buf.push(b's');
                pad_to_align(4, buf);
                write_string(&int, byteorder, buf);
            }
            message::HeaderField::Member(mem) => {
                buf.push(3);
                buf.push(1);
                buf.push(b's');
                pad_to_align(4, buf);
                write_string(&mem, byteorder, buf);
            }
            message::HeaderField::ErrorName(name) => {
                buf.push(4);
                buf.push(1);
                buf.push(b's');
                pad_to_align(4, buf);
                write_string(&name, byteorder, buf);
            }
            message::HeaderField::ReplySerial(rs) => {
                buf.push(5);
                buf.push(1);
                buf.push(b'u');
                pad_to_align(4, buf);
                write_u32(*rs, byteorder, buf);
            }
            message::HeaderField::Destination(dest) => {
                buf.push(6);
                buf.push(1);
                buf.push(b's');
                pad_to_align(4, buf);
                write_string(&dest, byteorder, buf);
            }
            message::HeaderField::Sender(snd) => {
                buf.push(7);
                buf.push(1);
                buf.push(b's');
                pad_to_align(4, buf);
                write_string(&snd, byteorder, buf);
            }
            message::HeaderField::Signature(sig) => {
                buf.push(8);
                buf.push(1);
                buf.push(b'g');
                write_string(&sig, byteorder, buf);
            }
            message::HeaderField::UnixFds(fds) => {
                buf.push(9);
                buf.push(1);
                buf.push(b'u');
                pad_to_align(4, buf);
                write_u32(*fds, byteorder, buf);
            }
        }
    }
}

fn marshal_base_param(p: &message::Base, buf: &mut Vec<u8>) -> message::Result<()> {
    match p {
        message::Base::Boolean(b) => {
            pad_to_align(4, buf);
            buf.push(0);
            buf.push(0);
            buf.push(0);
            if *b {
                buf.push(0);
            } else {
                buf.push(1);
            }
            Ok(())
        }
        message::Base::Int32(i) => {
            pad_to_align(4, buf);
            buf.push((*i >> 0) as u8);
            buf.push((*i >> 8) as u8);
            buf.push((*i >> 16) as u8);
            buf.push((*i >> 24) as u8);
            Ok(())
        }
        message::Base::Uint32(i) => {
            let raw = *i as u32;
            pad_to_align(4, buf);
            buf.push((raw >> 0) as u8);
            buf.push((raw >> 8) as u8);
            buf.push((raw >> 16) as u8);
            buf.push((raw >> 24) as u8);
            Ok(())
        }
        message::Base::String(s) => {
            pad_to_align(4, buf);
            let len = s.len() as u32;
            buf.push((len >> 0) as u8);
            buf.push((len >> 8) as u8);
            buf.push((len >> 16) as u8);
            buf.push((len >> 24) as u8);
            buf.extend(s.bytes());
            buf.push(0);
            Ok(())
        }
        message::Base::Signature(s) => {
            message::validate_signature(&s)?;
            pad_to_align(1, buf);
            let len = s.len() as u8;
            buf.push(len);
            buf.extend(s.bytes());
            buf.push(0);
            Ok(())
        }
        message::Base::ObjectPath(s) => {
            message::validate_object_path(&s)?;
            pad_to_align(4, buf);
            let len = s.len() as u32;
            buf.push((len >> 0) as u8);
            buf.push((len >> 8) as u8);
            buf.push((len >> 16) as u8);
            buf.push((len >> 24) as u8);
            buf.extend(s.bytes());
            buf.push(0);
            Ok(())
        }
    }
}

fn marshal_container_param(p: &message::Container, buf: &mut Vec<u8>) -> message::Result<()> {
    match p {
        message::Container::Array(params) => {
            message::validate_array(&params)?;
            pad_to_align(4, buf);
            let len = params.len() as u32;
            buf.push((len >> 0) as u8);
            buf.push((len >> 8) as u8);
            buf.push((len >> 16) as u8);
            buf.push((len >> 24) as u8);
            for p in params {
                marshal_param(&p, buf)?;
            }
        }
        message::Container::Struct(params) => {
            pad_to_align(8, buf);
            for p in params {
                marshal_param(&p, buf)?;
            }
        }
        message::Container::DictEntry(key, value) => {
            pad_to_align(8, buf);
            marshal_base_param(&key, buf)?;
            marshal_param(&value, buf)?;
        }
        message::Container::Variant(variant) => {
            let mut sig_str = String::new();
            variant.sig.to_str(&mut sig_str);
            buf.push(sig_str.len() as u8);
            buf.extend(sig_str.bytes());
            marshal_param(&variant.value, buf)?;
        }
    }
    Ok(())
}

fn marshal_param(p: &message::Param, buf: &mut Vec<u8>) -> message::Result<()> {
    match p {
        message::Param::Base(b) => marshal_base_param(&b, buf),
        message::Param::Container(c) => marshal_container_param(&c, buf),
    }
}
