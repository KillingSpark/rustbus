//! All things relevant to unmarshalling content from raw bytes
//!
//! * `base` and `container` are for the Param approach that map dbus concepts to enums/structs
//! * `traits` is for the trait based approach
//! * `iter` is an experimental approach to an libdbus-like iterator

use std::num::NonZeroU32;

use crate::message_builder::DynamicHeader;
use crate::message_builder::MarshalledMessage;
use crate::message_builder::MarshalledMessageBody;
use crate::message_builder::MessageType;
use crate::params;
use crate::signature;
use crate::wire::errors::UnmarshalError;
use crate::wire::util::*;
use crate::wire::HeaderField;
use crate::ByteOrder;

mod param;
pub use param::base;
pub use param::container;
pub mod iter;
pub mod traits;

use container::*;

use super::unmarshal_context::{Cursor, UnmarshalContext};
use super::UnixFd;

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub byteorder: ByteOrder,
    pub typ: MessageType,
    pub flags: u8,
    pub version: u8,
    pub body_len: u32,
    pub serial: NonZeroU32,
}

impl From<crate::signature::Error> for UnmarshalError {
    fn from(e: crate::signature::Error) -> Self {
        UnmarshalError::Validation(crate::params::validation::Error::InvalidSignature(e))
    }
}

impl UnmarshalError {
    /// Checks if `self` is an `EndOfMessage` error.
    #[inline]
    pub fn is_end_of_message(&self) -> bool {
        self == &UnmarshalError::EndOfMessage
    }
}

pub type UnmarshalResult<T> = std::result::Result<T, UnmarshalError>;

pub const HEADER_LEN: usize = 12;

pub fn unmarshal_header(cursor: &mut Cursor) -> UnmarshalResult<Header> {
    if cursor.remainder().len() < HEADER_LEN {
        return Err(UnmarshalError::NotEnoughBytes);
    }

    let byteorder = match cursor.read_u8()? {
        b'l' => ByteOrder::LittleEndian,
        b'B' => ByteOrder::BigEndian,
        _ => return Err(UnmarshalError::InvalidByteOrder),
    };

    let typ = match cursor.read_u8()? {
        1 => MessageType::Call,
        2 => MessageType::Reply,
        3 => MessageType::Error,
        4 => MessageType::Signal,
        _ => return Err(UnmarshalError::InvalidMessageType),
    };
    let flags = cursor.read_u8()?;
    let version = cursor.read_u8()?;
    let body_len = cursor.read_u32(byteorder)?;
    let serial =
        NonZeroU32::new(cursor.read_u32(byteorder)?).ok_or(UnmarshalError::InvalidSerial)?;

    Ok(Header {
        byteorder,
        typ,
        flags,
        version,
        body_len,
        serial,
    })
}

pub fn unmarshal_dynamic_header(
    header: &Header,
    cursor: &mut Cursor,
) -> UnmarshalResult<DynamicHeader> {
    let fields = unmarshal_header_fields(header, cursor)?;
    let mut hdr = DynamicHeader {
        serial: Some(header.serial),
        ..Default::default()
    };
    collect_header_fields(&fields, &mut hdr);
    Ok(hdr)
}

pub fn unmarshal_body(
    byteorder: ByteOrder,
    sigs: &[crate::signature::Type],
    buf: &[u8],
    fds: &[crate::wire::UnixFd],
    offset: usize,
) -> UnmarshalResult<Vec<params::Param<'static, 'static>>> {
    let mut params = Vec::new();
    let mut ctx = UnmarshalContext::new(fds, byteorder, buf, offset);
    for param_sig in sigs {
        let new_param = unmarshal_with_sig(param_sig, &mut ctx)?;
        params.push(new_param);
    }
    Ok(params)
}

pub fn unmarshal_next_message(
    header: &Header,
    dynheader: DynamicHeader,
    buf: Vec<u8>,
    offset: usize,
    raw_fds: Vec<UnixFd>,
) -> UnmarshalResult<MarshalledMessage> {
    let sig = dynheader.signature.clone().unwrap_or_else(|| "".to_owned());
    let padding = align_offset(8, &buf, offset)?;

    if header.body_len == 0 {
        let msg = MarshalledMessage {
            dynheader,
            body: MarshalledMessageBody::from_parts(vec![], 0, raw_fds, sig, header.byteorder),
            typ: header.typ,
            flags: header.flags,
        };
        Ok(msg)
    } else {
        let offset = offset + padding;

        if buf[offset..].len() < (header.body_len as usize) {
            return Err(UnmarshalError::NotEnoughBytes);
        }
        if buf[offset..].len() != header.body_len as usize {
            return Err(UnmarshalError::NotAllBytesUsed);
        }

        let msg = MarshalledMessage {
            dynheader,
            body: MarshalledMessageBody::from_parts(buf, offset, raw_fds, sig, header.byteorder),
            typ: header.typ,
            flags: header.flags,
        };
        Ok(msg)
    }
}

fn unmarshal_header_fields(
    header: &Header,
    cursor: &mut Cursor,
) -> UnmarshalResult<Vec<HeaderField>> {
    let header_fields_bytes = cursor.read_u32(header.byteorder)?;

    if cursor.remainder().len() < header_fields_bytes as usize {
        return Err(UnmarshalError::NotEnoughBytes);
    }

    let mut cursor = Cursor::new(cursor.read_raw(header_fields_bytes as usize)?);
    let mut fields = Vec::new();

    while !cursor.remainder().is_empty() {
        match unmarshal_header_field(header, &mut cursor) {
            Ok(field) => {
                fields.push(field);
            }
            Err(UnmarshalError::UnknownHeaderField) => {
                // try to validate that there is indeed a valid dbus variant. This is mandatory so the message follows the spec,
                // even if we just ignore the contents.
                match crate::wire::validate_raw::validate_marshalled(
                    header.byteorder,
                    0,
                    cursor.remainder(),
                    &crate::signature::Type::Container(crate::signature::Container::Variant),
                ) {
                    Ok(bytes) => {
                        // ignore happy path, but increase counter.
                        cursor.advance(bytes);
                    }
                    // if the unknown header contains invalid values this is still an error, and the message should be treated as unreadable
                    Err((_bytes, err)) => return Err(err),
                }
            }
            Err(e) => return Err(e),
        }
    }
    params::validate_header_fields(header.typ, &fields)
        .map_err(|_| UnmarshalError::InvalidHeaderFields)?;

    Ok(fields)
}

fn unmarshal_header_field(header: &Header, cursor: &mut Cursor) -> UnmarshalResult<HeaderField> {
    // align to 8 because the header fields are an array of structs `a(yv)`
    cursor.align_to(8)?;

    let typ = cursor.read_u8()?;

    let sig_str = cursor.read_signature()?;
    let mut sig =
        signature::Type::parse_description(sig_str).map_err(|_| UnmarshalError::NoSignature)?;

    if sig.len() != 1 {
        // There must be exactly one type in the signature!
        return Err(UnmarshalError::NoSignature);
    }
    let sig = sig.remove(0);
    match typ {
        1 => match sig {
            signature::Type::Base(signature::Base::ObjectPath) => {
                let objpath = cursor.read_str(header.byteorder)?;
                crate::params::validate_object_path(objpath)?;
                Ok(HeaderField::Path(objpath.to_owned()))
            }
            _ => Err(UnmarshalError::WrongSignature),
        },
        2 => match sig {
            signature::Type::Base(signature::Base::String) => Ok(HeaderField::Interface(
                cursor.read_str(header.byteorder)?.to_owned(),
            )),
            _ => Err(UnmarshalError::WrongSignature),
        },
        3 => match sig {
            signature::Type::Base(signature::Base::String) => Ok(HeaderField::Member(
                cursor.read_str(header.byteorder)?.to_owned(),
            )),
            _ => Err(UnmarshalError::WrongSignature),
        },
        4 => match sig {
            signature::Type::Base(signature::Base::String) => Ok(HeaderField::ErrorName(
                cursor.read_str(header.byteorder)?.to_owned(),
            )),
            _ => Err(UnmarshalError::WrongSignature),
        },
        5 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                NonZeroU32::new(cursor.read_u32(header.byteorder)?)
                    .ok_or(UnmarshalError::InvalidHeaderField)
                    .map(HeaderField::ReplySerial)
            }
            _ => Err(UnmarshalError::WrongSignature),
        },
        6 => match sig {
            signature::Type::Base(signature::Base::String) => Ok(HeaderField::Destination(
                cursor.read_str(header.byteorder)?.to_owned(),
            )),
            _ => Err(UnmarshalError::WrongSignature),
        },
        7 => match sig {
            signature::Type::Base(signature::Base::String) => Ok(HeaderField::Sender(
                cursor.read_str(header.byteorder)?.to_owned(),
            )),
            _ => Err(UnmarshalError::WrongSignature),
        },
        8 => match sig {
            signature::Type::Base(signature::Base::Signature) => {
                let sig = cursor.read_signature()?;
                // empty signature is allowed here
                if !sig.is_empty() {
                    crate::params::validate_signature(sig)?;
                }
                Ok(HeaderField::Signature(sig.to_owned()))
            }
            _ => Err(UnmarshalError::WrongSignature),
        },
        9 => match sig {
            signature::Type::Base(signature::Base::Uint32) => {
                Ok(HeaderField::UnixFds(cursor.read_u32(header.byteorder)?))
            }
            _ => Err(UnmarshalError::WrongSignature),
        },
        0 => Err(UnmarshalError::InvalidHeaderField),
        _ => Err(UnmarshalError::UnknownHeaderField),
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
