use crate::message;
use crate::signature;

pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
}

#[derive(Clone, Copy)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

pub enum HeaderFlags {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

pub enum HeaderField {
    Path(String),
    Interface(String),
    Member(String),
    ErrorName(String),
    ReplySerial(u32),
    Destination(String),
    Sender(String),
    Signature(String),
    UnixFds(u32),
}

type Result<T> = std::result::Result<T, Error>;

pub fn validate_object_path(_op: &str) -> Result<()> {
    // TODO
    Ok(())
}
pub fn validate_signature(sig: &str) -> Result<()> {
    if signature::Type::from_str(sig).is_err() {
        Err(Error::InvalidSignature)
    } else {
        Ok(())
    }
}

pub fn validate_array(_array: &Vec<message::Param>) -> Result<()> {
    // TODO check that all elements have the same type
    Ok(())
}

pub fn marshal(
    msg: &message::Message,
    byteorder: ByteOrder,
    serial: u32,
    header_fields: &Vec<HeaderField>,
    buf: &mut Vec<u8>,
) -> Result<()> {
    marshal_header(msg, byteorder, serial, header_fields, buf)?;
    let header_len = buf.len();
    match msg {
        message::Message::Reply => unimplemented!(),
        message::Message::Signal => unimplemented!(),
        message::Message::Error => unimplemented!(),
        message::Message::Call(c) => {
            // TODO marshal interface and member
            for p in &c.params {
                marshal_param(p, buf)?;
            }
        }
    }

    // set the correct message length
    let body_len = buf.len() - header_len;
    match byteorder {
        ByteOrder::LittleEndian => {
            buf[4] = (body_len >> 0) as u8;
            buf[5] = (body_len >> 8) as u8;
            buf[6] = (body_len >> 16) as u8;
            buf[7] = (body_len >> 24) as u8;
        }
        ByteOrder::BigEndian => {
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

fn write_u32(val: u32, byteorder: ByteOrder, buf: &mut Vec<u8>) {
    match byteorder {
        ByteOrder::LittleEndian => {
            buf.push((val >> 0) as u8);
            buf.push((val >> 8) as u8);
            buf.push((val >> 16) as u8);
            buf.push((val >> 24) as u8);
        }
        ByteOrder::BigEndian => {
            buf.push((val >> 24) as u8);
            buf.push((val >> 16) as u8);
            buf.push((val >> 8) as u8);
            buf.push((val >> 0) as u8);
        }
    }
}

fn write_string(val: &str, byteorder: ByteOrder, buf: &mut Vec<u8>) {
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
    byteorder: ByteOrder,
    serial: u32,
    header_fields: &Vec<HeaderField>,
    buf: &mut Vec<u8>,
) -> Result<()> {
    match byteorder {
        ByteOrder::BigEndian => {
            buf.push(b'B');
        }
        ByteOrder::LittleEndian => {
            buf.push(b'l');
        }
    }

    let msg_type = match msg {
        message::Message::Call(_) => 1,
        message::Message::Reply => 2,
        message::Message::Error => 3,
        message::Message::Signal => 4,
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
    byteorder: ByteOrder,
    header_fields: &Vec<HeaderField>,
    buf: &mut Vec<u8>,
) {
    for field in header_fields {
        match field {
            HeaderField::Path(path) => {
                buf.push(1);
                pad_to_align(4, buf);
                write_string(&path, byteorder, buf);
            }
            HeaderField::Interface(int) => {
                buf.push(2);
                pad_to_align(4, buf);
                write_string(&int, byteorder, buf);
            }
            HeaderField::Member(mem) => {
                buf.push(3);
                pad_to_align(4, buf);
                write_string(&mem, byteorder, buf);
            }
            HeaderField::ErrorName(name) => {
                buf.push(4);
                pad_to_align(4, buf);
                write_string(&name, byteorder, buf);
            }
            HeaderField::ReplySerial(rs) => {
                buf.push(4);
                pad_to_align(4, buf);
                write_u32(*rs, byteorder, buf);
            }
            HeaderField::Destination(dest) => {
                buf.push(4);
                pad_to_align(4, buf);
                write_string(&dest, byteorder, buf);
            }
            HeaderField::Sender(snd) => {
                buf.push(4);
                pad_to_align(4, buf);
                write_string(&snd, byteorder, buf);
            }
            HeaderField::Signature(sig) => {
                buf.push(4);
                pad_to_align(4, buf);
                write_string(&sig, byteorder, buf);
            }
            HeaderField::UnixFds(fds) => {
                buf.push(4);
                pad_to_align(4, buf);
                write_u32(*fds, byteorder, buf);
            }
        }
    }
}

fn marshal_base_param(p: &message::Base, buf: &mut Vec<u8>) -> Result<()> {
    // TODO padding
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
            validate_signature(&s)?;
            pad_to_align(1, buf);
            let len = s.len() as u8;
            buf.push(len);
            buf.extend(s.bytes());
            buf.push(0);
            Ok(())
        }
        message::Base::ObjectPath(s) => {
            validate_object_path(&s)?;
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

fn marshal_container_param(p: &message::Container, buf: &mut Vec<u8>) -> Result<()> {
    match p {
        message::Container::Array(params) => {
            validate_array(&params)?;
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
    }
    Ok(())
}

fn marshal_param(p: &message::Param, buf: &mut Vec<u8>) -> Result<()> {
    match p {
        message::Param::Base(b) => marshal_base_param(&b, buf),
        message::Param::Container(c) => marshal_container_param(&c, buf),
    }
}
