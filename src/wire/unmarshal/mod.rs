//! All things relevant to unmarshalling content from raw bytes
//!
//! * `base` and `container` are for the Param approach that map dbus concepts to enums/structs
//! * `traits` is for the trait based approach
//! * `iter` is an experimental approach to an libdbus-like iterator

use crate::message_builder::DynamicHeader;
use crate::message_builder::MarshalledMessage;
use crate::message_builder::MarshalledMessageBody;
use crate::message_builder::MessageType;
use crate::params;
use crate::signature;
use crate::wire::util::*;
use crate::wire::HeaderField;
use crate::ByteOrder;

pub mod base;
pub mod container;
pub mod iter;
pub mod traits;

use container::*;

#[derive(Debug)]
pub struct Header {
    pub byteorder: ByteOrder,
    pub typ: MessageType,
    pub flags: u8,
    pub version: u8,
    pub body_len: u32,
    pub serial: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NotEnoughBytes,
    NotEnoughBytesForCollection,
    NotAllBytesUsed,
    InvalidByteOrder,
    InvalidType,
    WrongSignature,
    Validation(crate::params::validation::Error),
    InvalidHeaderField,
    InvalidHeaderFields,
    UnknownHeaderField,
    PaddingContainedData,
    InvalidBoolean,
    EndOfMessage,
    NoSerial,
    NoSignature,
}

impl From<crate::params::validation::Error> for Error {
    fn from(e: crate::params::validation::Error) -> Self {
        Error::Validation(e)
    }
}
impl From<crate::signature::Error> for Error {
    fn from(e: crate::signature::Error) -> Self {
        Error::Validation(crate::params::validation::Error::InvalidSignature(e))
    }
}

impl Error {
    /// Checks if `self` is an `EndOfMessage` error.
    #[inline]
    pub fn is_end_of_message(&self) -> bool {
        self == &Error::EndOfMessage
    }
}

pub const HEADER_LEN: usize = 12;

pub type UnmarshalResult<T> = std::result::Result<(usize, T), Error>;

pub fn unmarshal_header(buf: &[u8], offset: usize) -> UnmarshalResult<Header> {
    if buf.len() < offset + HEADER_LEN {
        return Err(Error::NotEnoughBytes);
    }
    let header_slice = &buf[offset..offset + HEADER_LEN];

    let byteorder = match header_slice[0] {
        b'l' => ByteOrder::LittleEndian,
        b'B' => ByteOrder::BigEndian,
        _ => return Err(Error::InvalidByteOrder),
    };

    let typ = match header_slice[1] {
        1 => MessageType::Call,
        2 => MessageType::Reply,
        3 => MessageType::Error,
        4 => MessageType::Signal,
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

pub fn unmarshal_dynamic_header(
    header: &Header,
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<DynamicHeader> {
    let (fields_bytes_used, fields) = unmarshal_header_fields(header, buf, offset)?;
    let mut hdr = DynamicHeader::default();
    hdr.serial = Some(header.serial);
    collect_header_fields(&fields, &mut hdr);
    Ok((fields_bytes_used, hdr))
}

pub fn unmarshal_body<'a, 'e>(
    byteorder: ByteOrder,
    sigs: &[crate::signature::Type],
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<Vec<params::Param<'a, 'e>>> {
    let mut params = Vec::new();
    let mut body_bytes_used = 0;
    for param_sig in sigs {
        let (bytes, new_param) =
            unmarshal_with_sig(byteorder, &param_sig, buf, offset + body_bytes_used)?;
        params.push(new_param);
        body_bytes_used += bytes;
    }
    Ok((body_bytes_used, params))
}

pub fn unmarshal_next_message(
    header: &Header,
    dynheader: DynamicHeader,
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<MarshalledMessage> {
    let sig = dynheader.signature.clone().unwrap_or_else(|| "".to_owned());

    if header.body_len == 0 {
        let padding = align_offset(8, buf, offset)?;
        let msg = MarshalledMessage {
            dynheader,
            body: MarshalledMessageBody::from_parts(vec![], sig, header.byteorder),
            typ: header.typ,
            raw_fds: Vec::new(),
            flags: header.flags,
        };
        Ok((padding, msg))
    } else {
        let padding = align_offset(8, buf, offset)?;
        let offset = offset + padding;

        if buf[offset..].len() < (header.body_len as usize) {
            return Err(Error::NotEnoughBytes);
        }

        let msg = MarshalledMessage {
            dynheader,
            body: MarshalledMessageBody::from_parts(buf[offset..].to_vec(), sig, header.byteorder),
            typ: header.typ,
            raw_fds: Vec::new(),
            flags: header.flags,
        };
        Ok((padding + header.body_len as usize, msg))
    }
}

fn unmarshal_header_fields(
    header: &Header,
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<Vec<HeaderField>> {
    let (_, header_fields_bytes) = parse_u32(&buf[offset..], header.byteorder)?;
    let offset = offset + 4;

    let mut fields = Vec::new();
    let mut bytes_used_counter = 0;

    while bytes_used_counter < header_fields_bytes as usize {
        match unmarshal_header_field(header, buf, offset + bytes_used_counter) {
            Ok((bytes_used, field)) => {
                fields.push(field);
                bytes_used_counter += bytes_used;
            }
            Err(Error::UnknownHeaderField) => {
                // ignore
            }
            Err(e) => return Err(e),
        }
    }
    params::validate_header_fields(header.typ, &fields).map_err(|_| Error::InvalidHeaderFields)?;

    Ok((header_fields_bytes as usize + 4, fields))
}

fn unmarshal_header_field(
    header: &Header,
    buf: &[u8],
    offset: usize,
) -> UnmarshalResult<HeaderField> {
    let padding = align_offset(8, buf, offset)?;
    let offset = offset + padding;

    if buf.is_empty() {
        return Err(Error::NotEnoughBytes);
    }
    let typ = buf[offset];
    let typ_bytes_used = 1;
    let offset = offset + typ_bytes_used;

    let (sig_bytes_used, sig_str) = unmarshal_signature(&buf[offset..])?;
    let mut sig = signature::Type::parse_description(&sig_str).map_err(|_| Error::NoSignature)?;
    let offset = offset + sig_bytes_used;

    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(Error::NoSignature);
    }
    let sig = sig.remove(0);
    let (field_bytes_used, field) = match typ {
        1 => match sig {
            signature::Type::Base(signature::Base::ObjectPath) => {
                let (b, objpath) = unmarshal_string(header.byteorder, &buf[offset..])?;
                // TODO validate
                (b, Ok(HeaderField::Path(objpath)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        2 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, int) = unmarshal_string(header.byteorder, &buf[offset..])?;
                (b, Ok(HeaderField::Interface(int)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        3 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, mem) = unmarshal_string(header.byteorder, &buf[offset..])?;
                (b, Ok(HeaderField::Member(mem)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        4 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, name) = unmarshal_string(header.byteorder, &buf[offset..])?;
                (b, Ok(HeaderField::ErrorName(name)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        5 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                let (b, serial) = parse_u32(&buf[offset..], header.byteorder)?;
                (b, Ok(HeaderField::ReplySerial(serial)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        6 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, dest) = unmarshal_string(header.byteorder, &buf[offset..])?;
                (b, Ok(HeaderField::Destination(dest)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        7 => match sig {
            signature::Type::Base(signature::Base::String) => {
                let (b, snd) = unmarshal_string(header.byteorder, &buf[offset..])?;
                (b, Ok(HeaderField::Sender(snd)))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        8 => match sig {
            signature::Type::Base(signature::Base::Signature) => {
                let (b, sig) = unmarshal_signature(&buf[offset..])?;
                (b, Ok(HeaderField::Signature(sig.to_owned())))
            }
            _ => (0, Err(Error::WrongSignature)),
        },
        9 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                let (b, fds) = parse_u32(&buf[offset..], header.byteorder)?;
                (b, Ok(HeaderField::UnixFds(fds)))
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

fn collect_header_fields(header_fields: &[HeaderField], hdr: &mut DynamicHeader) {
    for h in header_fields {
        match h {
            HeaderField::Destination(d) => hdr.destination = Some(d.clone()),
            HeaderField::ErrorName(e) => hdr.error_name = Some(e.clone()),
            HeaderField::Interface(s) => hdr.interface = Some(s.clone()),
            HeaderField::Member(m) => hdr.member = Some(m.clone()),
            HeaderField::Path(p) => hdr.object = Some(p.clone()),
            HeaderField::ReplySerial(r) => hdr.response_serial = Some(*r),
            HeaderField::Sender(s) => hdr.sender = Some(s.clone()),
            HeaderField::Signature(s) => hdr.signature = Some(s.clone()),
            HeaderField::UnixFds(u) => hdr.num_fds = Some(*u),
        }
    }
}
