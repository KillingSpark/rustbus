use crate::message;
use crate::signature;

pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
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

pub fn marshal(msg: &message::Message, buf: &mut Vec<u8>) -> Result<()> {
    match msg {
        message::Message::Reply => unimplemented!(),
        message::Message::Signal => unimplemented!(),
        message::Message::Call(c) => {
            // TODO marshal interface and member
            for p in &c.params {
                marshal_param(p, buf)?;
            }
            Ok(())
        }
    }
}

fn pad_to_align(align_to: usize, buf: &mut Vec<u8>) {
    let padding_needed = buf.len() % align_to;
    buf.resize(buf.len() + padding_needed, 0);
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
