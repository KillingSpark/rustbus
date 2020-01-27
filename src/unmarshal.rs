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

pub fn read_u64(buf: &mut Vec<u8>, byteorder: message::ByteOrder) -> Result<u64, Error> {
    if buf.len() < 8 {
        return Err(Error::NotEnoughBytes);
    }
    let number = buf.drain(..8).collect::<Vec<_>>();
    Ok(parse_u64(&number, byteorder)?.1)
}

pub fn parse_u64(number: &[u8], byteorder: message::ByteOrder) -> UnmarshalResult<u64> {
    if number.len() < 8 {
        return Err(Error::NotEnoughBytes);
    }
    let val = match byteorder {
        message::ByteOrder::LittleEndian => {
            (number[0] as u64)
                + ((number[1] as u64) << 8)
                + ((number[2] as u64) << 16)
                + ((number[3] as u64) << 24)
                + ((number[4] as u64) << 32)
                + ((number[5] as u64) << 40)
                + ((number[6] as u64) << 48)
                + ((number[7] as u64) << 56)
        }
        message::ByteOrder::BigEndian => {
            (number[7] as u64)
                + ((number[6] as u64) << 8)
                + ((number[5] as u64) << 16)
                + ((number[4] as u64) << 24)
                + ((number[3] as u64) << 32)
                + ((number[2] as u64) << 40)
                + ((number[1] as u64) << 48)
                + ((number[0] as u64) << 56)
        }
    };
    Ok((8, val))
}

pub fn parse_u32(number: &[u8], byteorder: message::ByteOrder) -> UnmarshalResult<u32> {
    if number.len() < 4 {
        return Err(Error::NotEnoughBytes);
    }
    let val = match byteorder {
        message::ByteOrder::LittleEndian => {
            (number[0] as u32)
                + ((number[1] as u32) << 8)
                + ((number[2] as u32) << 16)
                + ((number[3] as u32) << 24)
        }
        message::ByteOrder::BigEndian => {
            (number[3] as u32)
                + ((number[2] as u32) << 8)
                + ((number[1] as u32) << 16)
                + ((number[0] as u32) << 24)
        }
    };
    Ok((4, val))
}

pub fn read_u32(buf: &mut Vec<u8>, byteorder: message::ByteOrder) -> Result<u32, Error> {
    if buf.len() < 4 {
        return Err(Error::NotEnoughBytes);
    }
    let number = buf.drain(..4).collect::<Vec<_>>();
    let (_, val) = parse_u32(&number, byteorder)?;
    Ok(val)
}

pub fn parse_u16(number: &[u8], byteorder: message::ByteOrder) -> UnmarshalResult<u16> {
    if number.len() < 2 {
        return Err(Error::NotEnoughBytes);
    }
    let val = match byteorder {
        message::ByteOrder::LittleEndian => (number[0] as u16) + ((number[1] as u16) << 8),
        message::ByteOrder::BigEndian => (number[1] as u16) + ((number[0] as u16) << 8),
    };
    Ok((2, val))
}

pub fn read_u16(buf: &mut Vec<u8>, byteorder: message::ByteOrder) -> Result<u16, Error> {
    if buf.len() < 2 {
        return Err(Error::NotEnoughBytes);
    }
    let number = buf.drain(..2).collect::<Vec<_>>();
    Ok(parse_u16(&number, byteorder)?.1)
}

type UnmarshalResult<T> = std::result::Result<(usize, T), Error>;

pub fn unmarshal_header(buf: &Vec<u8>, offset: usize) -> UnmarshalResult<Header> {
    if buf.len() < offset + HEADER_LEN {
        return Err(Error::NotEnoughBytes);
    }
    let header_slice = &buf[offset..offset + HEADER_LEN];

    let byteorder = match header_slice[0] {
        b'l' => message::ByteOrder::LittleEndian,
        b'B' => message::ByteOrder::BigEndian,
        _ => return Err(Error::InvalidByteOrder),
    };

    let typ = match header_slice[1] {
        1 => message::MessageType::Call,
        2 => message::MessageType::Reply,
        3 => message::MessageType::Error,
        4 => message::MessageType::Signal,
        _ => return Err(Error::InvalidType),
    };
    let flags = header_slice[2];
    let version = header_slice[3];
    let (_, body_len) = parse_u32(&header_slice[4..8], byteorder)?;
    let (_, serial) = parse_u32(&header_slice[8..12], byteorder)?;

    Ok((
        HEADER_LEN,
        Header {
            byteorder,
            typ,
            flags,
            version,
            body_len,
            serial,
        },
    ))
}

pub fn unmarshal_next_message(
    header: &Header,
    buf: &mut Vec<u8>,
    offset: usize,
) -> UnmarshalResult<message::Message> {
    println!("Start reading header fields");
    let (fields_bytes_used, fields) = unmarshal_header_fields(header, buf, offset)?;
    println!(
        "Finished reading header fields: {} bytes",
        fields_bytes_used
    );
    let offset = offset + fields_bytes_used;

    // TODO find in fields
    if header.body_len == 0 {
        let padding = align_offset(8, buf, offset)?;
        Ok((
            padding + fields_bytes_used,
            message::Message {
                interface: get_interface_from_fields(&fields),
                member: get_member_from_fields(&fields),
                object: get_object_from_fields(&fields),
                destination: get_destination_from_fields(&fields),
                response_serial: get_resp_serial_from_fields(&fields),
                sender: get_sender_from_fields(&fields),
                error_name: get_errorname_from_fields(&fields),
                params: vec![],
                typ: header.typ,
                serial: Some(header.serial),
                raw_fds: Vec::new(),
                num_fds: get_unixfds_from_fields(&fields),
            },
        ))
    } else {
        println!("Need a signature");
        let sigs = match get_sig_from_fields(&fields) {
            Some(s) => signature::Type::from_str(&s).map_err(|_| Error::InvalidSignature)?,
            None => {
                // TODO this is ok if body_len == 0
                return Err(Error::InvalidHeaderFields);
            }
        };
        println!("Found a signature: {:?}", sigs);

        println!("Unmarshal body: {:?}", &buf[offset..]);
        let padding = align_offset(8, buf, offset)?;
        let offset = offset + padding;

        if buf[offset..].len() < (header.body_len as usize) {
            return Err(Error::NotEnoughBytes);
        }
        println!("Start reading params");
        let mut params = Vec::new();
        let mut body_bytes_used = 0;
        for param_sig in sigs {
            let (bytes, new_param) = unmarshal_with_sig(header, &param_sig, buf, offset)?;
            params.push(new_param);
            body_bytes_used += bytes;
        }
        Ok((
            padding + fields_bytes_used + body_bytes_used,
            message::Message {
                interface: get_interface_from_fields(&fields),
                member: get_member_from_fields(&fields),
                object: get_object_from_fields(&fields),
                destination: get_destination_from_fields(&fields),
                response_serial: get_resp_serial_from_fields(&fields),
                sender: get_sender_from_fields(&fields),
                error_name: get_errorname_from_fields(&fields),
                params: params,
                typ: header.typ,
                serial: Some(header.serial),
                raw_fds: Vec::new(),
                num_fds: get_unixfds_from_fields(&fields),
            },
        ))
    }
}

fn unmarshal_header_fields(
    header: &Header,
    buf: &Vec<u8>,
    offset: usize,
) -> UnmarshalResult<Vec<message::HeaderField>> {
    let (_, header_fields_bytes) = parse_u32(&buf[offset..], header.byteorder)?;
    let offset = offset + 4;

    let mut fields = Vec::new();
    let mut bytes_used_counter = 0;

    while bytes_used_counter < header_fields_bytes as usize {
        println!(
            "Bytes left in header fields: {}",
            header_fields_bytes as usize - bytes_used_counter
        );

        match unmarshal_header_field(header, buf, offset + bytes_used_counter) {
            Ok((bytes_used, field)) => {
                println!("Field: {:?}", field);
                fields.push(field);
                bytes_used_counter += bytes_used;
            }
            Err(Error::UnknownHeaderField) => {
                // ignore
            }
            Err(e) => return Err(e),
        }
    }
    message::validate_header_fields(header.typ, &fields).map_err(|_| Error::InvalidHeaderFields)?;

    Ok((header_fields_bytes as usize + 4, fields))
}

fn unmarshal_header_field(
    header: &Header,
    buf: &Vec<u8>,
    offset: usize,
) -> UnmarshalResult<message::HeaderField> {
    println!("before header field: {:?}", &buf[offset..]);
    let padding = align_offset(8, buf, offset)?;
    let offset = offset + padding;

    if buf.len() < 1 {
        return Err(Error::NotEnoughBytes);
    }
    let typ = buf[offset];
    let typ_bytes_used = 1;
    let offset = offset + typ_bytes_used;
    println!("TYPE: {}", typ);

    let (sig_bytes_used, sig_str) = unmarshal_signature(&buf[offset..])?;
    println!("Field sig: {}", sig_str);
    let mut sig = signature::Type::from_str(&sig_str).map_err(|_| Error::InvalidSignature)?;
    let offset = offset + sig_bytes_used;

    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(Error::InvalidSignature);
    }
    let sig = sig.remove(0);
    let (field_bytes_used, field) = match typ {
        1 => match sig {
            signature::Type::Base(signature::Base::ObjectPath) => {
                let (b, objpath) = unmarshal_string(header, &buf[offset..])?;
                // TODO validate
                (b, Ok(message::HeaderField::Path(objpath)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        2 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, int) = unmarshal_string(header, &buf[offset..])?;
                (b, Ok(message::HeaderField::Interface(int)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        3 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, mem) = unmarshal_string(header, &buf[offset..])?;
                (b, Ok(message::HeaderField::Member(mem)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        4 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, name) = unmarshal_string(header, &buf[offset..])?;
                (b, Ok(message::HeaderField::ErrorName(name)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        5 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                let (b, serial) = parse_u32(&buf[offset..], header.byteorder)?;
                (b, Ok(message::HeaderField::ReplySerial(serial)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        6 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, dest) = unmarshal_string(header, &buf[offset..])?;
                (b, Ok(message::HeaderField::Destination(dest)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        7 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, snd) = unmarshal_string(header, &buf[offset..])?;
                (b, Ok(message::HeaderField::Sender(snd)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        8 => match sig {
            signature::Type::Base(signature::Base::Signature) => {
                let (b, sig) = unmarshal_signature(&buf[offset..])?;
                (b, Ok(message::HeaderField::Signature(sig)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        9 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                let (b, fds) = parse_u32(&buf[offset..], header.byteorder)?;
                (b, Ok(message::HeaderField::UnixFds(fds)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        0 => (0, Err(Error::InvalidHeaderField)),
        _ => (0, Err(Error::UnknownHeaderField)),
    };
    let sum_bytes_used = padding + typ_bytes_used + sig_bytes_used + field_bytes_used;
    match field {
        Ok(field) => Ok((sum_bytes_used, field)),
        Err(e) => Err(e),
    }
}

fn unmarshal_with_sig(
    header: &Header,
    sig: &signature::Type,
    buf: &mut Vec<u8>,
    offset: usize,
) -> UnmarshalResult<message::Param> {
    println!("Unmarshal: {:?}", sig);
    println!("Unmarshal from: {:?}", buf);
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
    println!("param: {:?}", param);
    Ok((bytes, param))
}

fn unmarshal_variant(
    header: &Header,
    buf: &mut Vec<u8>,
    offset: usize,
) -> UnmarshalResult<message::Variant> {
    let (sig_bytes_used, sig_str) = unmarshal_signature(buf)?;
    let mut sig = signature::Type::from_str(&sig_str).map_err(|_| Error::InvalidSignature)?;
    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(Error::InvalidSignature);
    }
    let sig = sig.remove(0);
    let (param_bytes_used, param) = unmarshal_with_sig(header, &sig, buf, offset)?;
    Ok((
        sig_bytes_used + param_bytes_used,
        message::Variant { sig, value: param },
    ))
}

fn unmarshal_container(
    header: &Header,
    buf: &mut Vec<u8>,
    typ: &signature::Container,
    offset: usize,
) -> UnmarshalResult<message::Container> {
    let param = match typ {
        signature::Container::Array(elem_sig) => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let (_, bytes_in_array) = parse_u32(&buf[offset..], header.byteorder)?;
            let offset = offset + 4;

            let mut elements = Vec::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_array as usize {
                let (bytes_used, element) =
                    unmarshal_with_sig(header, &elem_sig, buf, offset + bytes_used_counter)?;
                elements.push(element);
                bytes_used_counter += bytes_used;
            }
            (
                padding + 4 + bytes_used_counter,
                message::Container::Array(elements),
            )
        }
        signature::Container::Dict(key_sig, val_sig) => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let bytes_in_dict = read_u32(buf, header.byteorder)?;
            let offset = offset + 4;

            let mut elements = std::collections::HashMap::new();
            let mut bytes_used_counter = 0;
            while bytes_used_counter < bytes_in_dict as usize {
                let (key_bytes, key) =
                    unmarshal_base(header, buf, *key_sig, offset + bytes_used_counter)?;
                bytes_used_counter += key_bytes;
                let (val_bytes, val) =
                    unmarshal_with_sig(header, val_sig, buf, offset + bytes_used_counter)?;
                bytes_used_counter += val_bytes;
                elements.insert(key, val);
            }
            (
                padding + 4 + bytes_used_counter,
                message::Container::Dict(elements),
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

fn unmarshal_base(
    header: &Header,
    buf: &Vec<u8>,
    typ: signature::Base,
    offset: usize,
) -> UnmarshalResult<message::Base> {
    match typ {
        signature::Base::Byte => {
            if buf.len() < 1 {
                return Err(Error::NotEnoughBytes);
            }
            Ok((1, message::Base::Byte(buf[offset])))
        }
        signature::Base::Uint16 => {
            let padding = align_offset(2, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 2];
            let (bytes, val) = parse_u16(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Uint16(val)))
        }
        signature::Base::Int16 => {
            let padding = align_offset(2, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 2];
            let (bytes, val) = parse_u16(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Int16(val as i16)))
        }
        signature::Base::Uint32 => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Uint32(val)))
        }
        signature::Base::UnixFd => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::UnixFd(val)))
        }
        signature::Base::Int32 => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Int32(val as i32)))
        }
        signature::Base::Uint64 => {
            let padding = align_offset(8, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Uint64(val)))
        }
        signature::Base::Int64 => {
            let padding = align_offset(8, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Int64(val as i64)))
        }
        signature::Base::Double => {
            let padding = align_offset(8, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 8];
            let (bytes, val) = parse_u64(slice, header.byteorder)?;
            Ok((bytes + padding, message::Base::Double(val)))
        }
        signature::Base::Boolean => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let slice = &buf[offset..offset + 4];
            let (bytes, val) = parse_u32(slice, header.byteorder)?;
            match val {
                0 => Ok((bytes + padding, message::Base::Boolean(false))),
                1 => Ok((bytes + padding, message::Base::Boolean(true))),
                _ => Err(Error::InvalidBoolean),
            }
        }
        signature::Base::String => {
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let (bytes, string) = unmarshal_string(header, &buf[offset..])?;
            Ok((bytes + padding, message::Base::String(string)))
        }
        signature::Base::ObjectPath => {
            // TODO validate
            let padding = align_offset(4, buf, offset)?;
            let offset = offset + padding;
            let (bytes, string) = unmarshal_string(header, &buf[offset..])?;
            Ok((bytes + padding, message::Base::ObjectPath(string)))
        }
        signature::Base::Signature => {
            // TODO validate
            let (bytes, string) = unmarshal_signature(buf)?;
            Ok((bytes, message::Base::Signature(string)))
        }
    }
}

fn align_offset(align_to: usize, buf: &Vec<u8>, offset: usize) -> Result<usize, Error> {
    let padding_delete = align_to - (offset % align_to);
    let padding_delete = if padding_delete == align_to {
        0
    } else {
        padding_delete
    };

    println!("Unpad: {}", padding_delete);

    if buf.len() < padding_delete {
        return Err(Error::NotEnoughBytes);
    }
    for x in 0..padding_delete {
        if buf[offset + x] != b'\0' {
            return Err(Error::PaddingContainedData);
        }
    }
    Ok(padding_delete)
}

fn unmarshal_signature(buf: &[u8]) -> UnmarshalResult<String> {
    if buf.len() < 1 {
        return Err(Error::NotEnoughBytes);
    }
    let len = buf[0] as usize;
    if buf.len() < len + 2 {
        return Err(Error::NotEnoughBytes);
    }
    let string = String::from_utf8(buf[1..len + 1].to_vec()).map_err(|_| Error::InvalidUtf8)?;
    Ok((len + 2, string))
}

fn unmarshal_string(header: &Header, buf: &[u8]) -> UnmarshalResult<String> {
    let len = parse_u32(buf, header.byteorder)?.1 as usize;
    if buf.len() < len + 4 {
        return Err(Error::NotEnoughBytes);
    }
    let string = String::from_utf8(buf[4..len + 4].to_vec()).map_err(|_| Error::InvalidUtf8)?;
    Ok((len + 5, string))
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

fn get_unixfds_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<u32> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(num_fds) => return Some(*num_fds),
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
fn get_resp_serial_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<u32> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(u) => return Some(*u),
            message::HeaderField::Sender(_) => {}
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}
fn get_sender_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(_) => {}
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(s) => return Some(s.clone()),
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}
fn get_errorname_from_fields(header_fields: &Vec<message::HeaderField>) -> Option<String> {
    for h in header_fields {
        match h {
            message::HeaderField::Destination(_) => {}
            message::HeaderField::ErrorName(s) => return Some(s.clone()),
            message::HeaderField::Interface(_) => {}
            message::HeaderField::Member(_) => {}
            message::HeaderField::Path(_) => {}
            message::HeaderField::ReplySerial(_) => {}
            message::HeaderField::Sender(_) => {},
            message::HeaderField::Signature(_) => {}
            message::HeaderField::UnixFds(_) => {}
        }
    }
    None
}
