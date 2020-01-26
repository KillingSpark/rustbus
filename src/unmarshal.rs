use crate::message;
use crate::signature;

#[derive(Debug)]
pub struct Header {
    pub byteorder: message::ByteOrder,
    pub typ: message::MessageType,
    pub flags: u8,
    pub version: u8,
    pub body_len: u32,
    pub serial: u32,
}

#[derive(Debug)]
pub enum Error {
    NotEnoughBytes,
    NotAllBytesUsed,
    InvalidByteOrder,
    InvalidType,
    InvalidSignature,
    WrongSignature,
    InvalidUtf8,
    InvalidHeaderField,
    InvalidHeaderFields,
    UnknownHeaderField,
    PaddingContainedData,
    InvalidBoolean,
}

pub const HEADER_LEN: usize = 12;

pub fn read_u32(buf: &mut Vec<u8>, byteorder: message::ByteOrder) -> Result<u32, Error> {
    if buf.len() < 4 {
        return Err(Error::NotEnoughBytes);
    }
    let number = buf.drain(..4).collect::<Vec<_>>();
    match byteorder {
        message::ByteOrder::LittleEndian => Ok((number[0] as u32)
            + ((number[1] as u32) << 8)
            + ((number[2] as u32) << 16)
            + ((number[3] as u32) << 24)),
        message::ByteOrder::BigEndian => Ok((number[3] as u32)
            + ((number[2] as u32) << 8)
            + ((number[1] as u32) << 16)
            + ((number[0] as u32) << 24)),
    }
}

fn read_i32(buf: &mut Vec<u8>, byteorder: message::ByteOrder) -> Result<i32, Error> {
    if buf.len() < 4 {
        return Err(Error::NotEnoughBytes);
    }
    let raw = read_u32(buf, byteorder)?;
    Ok(raw as i32)
}

pub fn unmarshal_header(buf: &mut Vec<u8>) -> Result<Header, Error> {
    if buf.len() < HEADER_LEN {
        return Err(Error::NotEnoughBytes);
    }
    let byteorder = match buf.remove(0) {
        b'l' => message::ByteOrder::LittleEndian,
        b'B' => message::ByteOrder::BigEndian,
        _ => return Err(Error::InvalidByteOrder),
    };
    let typ = match buf.remove(0) {
        1 => message::MessageType::Call,
        2 => message::MessageType::Reply,
        3 => message::MessageType::Error,
        4 => message::MessageType::Signal,
        _ => return Err(Error::InvalidType),
    };
    let flags = buf.remove(0);
    let version = buf.remove(0);
    let body_len = read_u32(buf, byteorder)?;
    let serial = read_u32(buf, byteorder)?;

    Ok(Header {
        byteorder,
        typ,
        flags,
        version,
        body_len,
        serial,
    })
}

pub fn unmarshal_next_message(
    header: &Header,
    buf: &mut Vec<u8>,
) -> Result<message::Message, Error> {
    let original_length = buf.len() + 12;

    println!("Start reading header fields");
    let fields = unmarshal_header_fields(header, buf, original_length)?;
    println!("Finished reading header fields");

    // TODO find in fields
    if header.body_len == 0 {
        unpad_to_align(8, buf, original_length)?;
        if !buf.is_empty() {
            return Err(Error::NotAllBytesUsed);
        }
        Ok(message::Message {
            interface: get_interface_from_fields(&fields),
            member: get_member_from_fields(&fields),
            object: get_object_from_fields(&fields),
            destination: get_destination_from_fields(&fields),
            params: vec![],
            typ: header.typ,
            serial: header.serial,
        })
    }else{
        println!("Need a signature");
        let sig = match get_sig_from_fields(&fields) {
            Some(s) => signature::Type::from_str(&s).map_err(|_| Error::InvalidSignature)?,
            None => {
                // TODO this is ok if body_len == 0
                return Err(Error::InvalidHeaderFields);
            }
        };
        println!("Found a signature: {:?}", sig);
    
        if buf.len() < header.body_len as usize {
            return Err(Error::NotEnoughBytes);
        }
    
        println!("Start reading params");
        unpad_to_align(8, buf, original_length)?;
        let params = unmarshal_with_sig(header, &sig, buf, original_length)?;
    
        if buf.len() != 0 {
            return Err(Error::NotAllBytesUsed);
        }
    
        Ok(message::Message {
            interface: get_interface_from_fields(&fields),
            member: get_member_from_fields(&fields),
            object: get_object_from_fields(&fields),
            destination: get_destination_from_fields(&fields),
            params: vec![params],
            typ: header.typ,
            serial: header.serial,
        })
    }
}

fn unmarshal_header_fields(
    header: &Header,
    buf: &mut Vec<u8>,
    original_length: usize,
) -> Result<Vec<message::HeaderField>, Error> {
    let header_fields_bytes = read_u32(buf, header.byteorder)?;
    let mut fields = Vec::new();

    let bytes_in_buf = buf.len();
    loop {
        let bytes_used = bytes_in_buf - buf.len();
        println!(
            "Bytes used: {} bytes in header fields: {}",
            bytes_used, header_fields_bytes
        );
        if bytes_used == header_fields_bytes as usize {
            break;
        }

        match unmarshal_header_field(header, buf, original_length) {
            Ok(field) => {
                println!("Field: {:?}", field);
                fields.push(field)
            }
            Err(Error::UnknownHeaderField) => {
                // ignore
            }
            Err(e) => return Err(e),
        }
    }
    message::validate_header_fields(header.typ, &fields).map_err(|_| Error::InvalidHeaderFields)?;

    Ok(fields)
}

fn unmarshal_header_field(
    header: &Header,
    buf: &mut Vec<u8>,
    original_length: usize,
) -> Result<message::HeaderField, Error> {
    println!("before header field: {:?}", buf);
    unpad_to_align(8, buf, original_length)?;
    if buf.len() < 1 {
        return Err(Error::NotEnoughBytes);
    }
    let typ = buf.remove(0);
    println!("TYPE: {}", typ);
    let sig_str = unmarshal_signature(buf)?;
    println!("Field sig: {}", sig_str);
    let sig = signature::Type::from_str(&sig_str).map_err(|_| Error::InvalidSignature)?;
    match typ {
        1 => match sig {
            signature::Type::Base(signature::Base::ObjectPath) => {
                let objpath = unmarshal_string(header, buf)?;
                // TODO validate
                Ok(message::HeaderField::Path(objpath))
            }
            _ => Err(Error::WrongSignature),
        },
        2 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let int = unmarshal_string(header, buf)?;
                Ok(message::HeaderField::Interface(int))
            }
            _ => Err(Error::WrongSignature),
        },
        3 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let mem = unmarshal_string(header, buf)?;
                Ok(message::HeaderField::Member(mem))
            }
            _ => Err(Error::WrongSignature),
        },
        4 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let name = unmarshal_string(header, buf)?;
                Ok(message::HeaderField::ErrorName(name))
            }
            _ => Err(Error::WrongSignature),
        },
        5 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                let serial = read_u32(buf, header.byteorder)?;
                Ok(message::HeaderField::ReplySerial(serial))
            }
            _ => Err(Error::WrongSignature),
        },
        6 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let dest = unmarshal_string(header, buf)?;
                Ok(message::HeaderField::Destination(dest))
            }
            _ => Err(Error::WrongSignature),
        },
        7 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let snd = unmarshal_string(header, buf)?;
                Ok(message::HeaderField::Sender(snd))
            }
            _ => Err(Error::WrongSignature),
        },
        8 => match sig {
            signature::Type::Base(signature::Base::Signature) => {
                let sig = unmarshal_signature(buf)?;
                Ok(message::HeaderField::Signature(sig))
            }
            _ => Err(Error::WrongSignature),
        },
        9 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                let fds = read_u32(buf, header.byteorder)?;
                Ok(message::HeaderField::UnixFds(fds))
            }
            _ => Err(Error::WrongSignature),
        },
        0 => Err(Error::InvalidHeaderField),
        _ => Err(Error::UnknownHeaderField),
    }
}

fn unmarshal_with_sig(
    header: &Header,
    sig: &signature::Type,
    buf: &mut Vec<u8>,
    original_length: usize,
) -> Result<message::Param, Error> {
    println!("Unmarshal: {:?}", sig);
    println!("Unmarshal from: {:?}", buf);
    let param = match &sig {
        signature::Type::Base(base) => {
            message::Param::Base(unmarshal_base(header, buf, *base, original_length)?)
        }
        signature::Type::Container(cont) => {
            message::Param::Container(unmarshal_container(header, buf, cont, original_length)?)
        }
    };
    println!("param: {:?}", param);
    Ok(param)
}

fn unmarshal_variant(
    header: &Header,
    buf: &mut Vec<u8>,
    original_length: usize,
) -> Result<message::Variant, Error> {
    let sig_str = unmarshal_signature(buf)?;
    let sig = signature::Type::from_str(&sig_str).map_err(|_| Error::InvalidSignature)?;
    let param = unmarshal_with_sig(header, &sig, buf, original_length)?;
    Ok(message::Variant { sig, value: param })
}

fn unmarshal_container(
    header: &Header,
    buf: &mut Vec<u8>,
    typ: &signature::Container,
    original_length: usize,
) -> Result<message::Container, Error> {
    let param = match typ {
        signature::Container::Array(elem_sig) => {
            unpad_to_align(4, buf, original_length)?;
            let bytes_in_array = read_u32(buf, header.byteorder)?;
            let mut elements = Vec::new();
            let bytes_in_buf = buf.len();
            loop {
                let bytes_used = bytes_in_buf - buf.len();
                if bytes_used == bytes_in_array as usize {
                    break;
                }
                let element = unmarshal_with_sig(header, &elem_sig, buf, original_length)?;
                elements.push(element);
            }
            message::Container::Array(elements)
        }
        signature::Container::Dict(key_sig, val_sig) => {
            unpad_to_align(4, buf, original_length)?;
            let bytes_in_dict = read_u32(buf, header.byteorder)?;
            let mut elements = std::collections::HashMap::new();
            let bytes_in_buf = buf.len();
            loop {
                let bytes_used = bytes_in_buf - buf.len();
                if bytes_used == bytes_in_dict as usize {
                    break;
                }
                let key = unmarshal_base(header, buf, *key_sig, original_length)?;
                let val = unmarshal_with_sig(header, val_sig, buf, original_length)?;
                elements.insert(key, val);
            }
            message::Container::Dict(elements)
        }
        signature::Container::Struct(sigs) => {
            unpad_to_align(8, buf, original_length)?;
            let mut fields = Vec::new();
            for field_sig in sigs {
                let field = unmarshal_with_sig(header, field_sig, buf, original_length)?;
                fields.push(field);
            }
            message::Container::Struct(fields)
        }
        signature::Container::Variant => {
            message::Container::Variant(Box::new(unmarshal_variant(header, buf, original_length)?))
        }
    };
    Ok(param)
}

fn unmarshal_base(
    header: &Header,
    buf: &mut Vec<u8>,
    typ: signature::Base,
    original_length: usize,
) -> Result<message::Base, Error> {
    match typ {
        signature::Base::Uint32 => {
            unpad_to_align(4, buf, original_length)?;
            let val = read_u32(buf, header.byteorder)?;
            Ok(message::Base::Uint32(val))
        }
        signature::Base::Int32 => {
            unpad_to_align(4, buf, original_length)?;
            let val = read_i32(buf, header.byteorder)?;
            Ok(message::Base::Int32(val))
        }
        signature::Base::Boolean => {
            unpad_to_align(4, buf, original_length)?;
            let val = read_u32(buf, header.byteorder)?;
            match val {
                0 => Ok(message::Base::Boolean(false)),
                1 => Ok(message::Base::Boolean(true)),
                _ => Err(Error::InvalidBoolean),
            }
        }
        signature::Base::String => {
            unpad_to_align(4, buf, original_length)?;
            let string = unmarshal_string(header, buf)?;
            Ok(message::Base::String(string))
        }
        signature::Base::ObjectPath => {
            unpad_to_align(4, buf, original_length)?;
            // TODO validate
            let string = unmarshal_string(header, buf)?;
            Ok(message::Base::String(string))
        }
        signature::Base::Signature => {
            // TODO validate
            let string = unmarshal_signature(buf)?;
            Ok(message::Base::Signature(string))
        }
    }
}

fn unpad_to_align(align_to: usize, buf: &mut Vec<u8>, original_length: usize) -> Result<(), Error> {
    let bytes_read = original_length - buf.len();
    let padding_delete = align_to - (bytes_read % align_to);
    let padding_delete = if padding_delete == align_to {
        0
    } else {
        padding_delete
    };

    println!("Unpad: {}", padding_delete);

    if buf.len() < padding_delete {
        return Err(Error::NotEnoughBytes);
    }
    let padding = buf.drain(0..padding_delete).collect::<Vec<_>>();
    for x in padding {
        if x != 0 {
            return Err(Error::PaddingContainedData);
        }
    }
    Ok(())
}

fn unmarshal_signature(buf: &mut Vec<u8>) -> Result<String, Error> {
    if buf.len() < 1 {
        return Err(Error::NotEnoughBytes);
    }
    let len = buf.remove(0) as usize;
    if buf.len() < len+1 {
        return Err(Error::NotEnoughBytes);
    }
    let bytes = buf.drain(0..len).collect();
    // drop \0
    buf.remove(0);
    String::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
}

fn unmarshal_string(header: &Header, buf: &mut Vec<u8>) -> Result<String, Error> {
    let len = read_u32(buf, header.byteorder)? as usize;
    if buf.len() < len+1 {
        return Err(Error::NotEnoughBytes);
    }
    let bytes = buf.drain(0..len).collect();
    // drop \0
    buf.remove(0);
    String::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
}

fn get_sig_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(s) => return Some(s.clone()),
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}

fn get_interface_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(s) => return Some(s.clone()),
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}

fn get_member_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(s) => return Some(s.clone()),
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}
fn get_object_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(s) => return Some(s.clone()),
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}
fn get_destination_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(s) => return Some(s.clone()),
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}
