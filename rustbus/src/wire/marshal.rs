//! All things relevant to marshalling content into raw bytes
//!
//! * `base` and `container` are for the Param approach that map dbus concepts to enums/structs
//! * `traits` is for the trait based approach

use std::num::NonZeroU32;

use crate::message_builder;
use crate::params;
use crate::wire::HeaderField;
use crate::ByteOrder;

use crate::wire::util::*;

mod param;
pub use param::base;
pub use param::container;
pub mod traits;

type MarshalResult<T> = Result<T, crate::wire::errors::MarshalError>;

pub struct MarshalContext<'fds, 'buf> {
    pub fds: &'fds mut Vec<crate::wire::UnixFd>,
    pub buf: &'buf mut Vec<u8>,
    pub byteorder: ByteOrder,
}

impl MarshalContext<'_, '_> {
    #[inline(always)]
    pub fn align_to(&mut self, alignment: usize) {
        pad_to_align(alignment, self.buf);
    }
}

/// This only prepares the header and dynheader fields. To send a message you still need the original message
/// and use get_buf() to get to the contents
pub fn marshal(
    msg: &crate::message_builder::MarshalledMessage,
    chosen_serial: NonZeroU32,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    marshal_header(msg, chosen_serial, buf)?;
    pad_to_align(8, buf);

    // set the correct message length
    insert_u32(
        msg.body.byteorder(),
        msg.get_buf().len() as u32,
        &mut buf[4..8],
    );
    Ok(())
}

fn marshal_header(
    msg: &crate::message_builder::MarshalledMessage,
    chosen_serial: NonZeroU32,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    let byteorder = msg.body.byteorder();

    match byteorder {
        ByteOrder::BigEndian => {
            buf.push(b'B');
        }
        ByteOrder::LittleEndian => {
            buf.push(b'l');
        }
    }

    let msg_type = match msg.typ {
        message_builder::MessageType::Invalid => {
            return Err(crate::wire::errors::MarshalError::InvalidMessageType)
        }
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
    buf.extend_from_slice(&[0, 0, 0, 0]);

    write_u32(chosen_serial.get(), byteorder, buf);

    // Zero bytes where the length of the header fields will be put
    let pos = buf.len();
    buf.extend_from_slice(&[0, 0, 0, 0]);

    if let Some(serial) = &msg.dynheader.response_serial {
        marshal_header_field(byteorder, &HeaderField::ReplySerial(*serial), buf)?;
    }
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
    if !msg.body.get_fds().is_empty() {
        marshal_header_field(
            byteorder,
            &HeaderField::UnixFds(msg.body.get_fds().len() as u32),
            buf,
        )?;
    }

    if !msg.get_buf().is_empty() {
        let sig_str = msg.get_sig().to_owned();
        marshal_header_field(byteorder, &HeaderField::Signature(sig_str), buf)?;
    }
    let len = buf.len() - pos - 4; // -4 the bytes for the length indicator do not count
    insert_u32(byteorder, len as u32, &mut buf[pos..pos + 4]);

    Ok(())
}

fn marshal_header_field(
    byteorder: ByteOrder,
    field: &HeaderField,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    pad_to_align(8, buf);
    match field {
        HeaderField::Path(path) => {
            params::validate_object_path(path)?;
            buf.push(1);
            buf.push(1);
            buf.push(b'o');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(path, byteorder, buf);
        }
        HeaderField::Interface(int) => {
            params::validate_interface(int)?;
            buf.push(2);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(int, byteorder, buf);
        }
        HeaderField::Member(mem) => {
            params::validate_membername(mem)?;
            buf.push(3);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(mem, byteorder, buf);
        }
        HeaderField::ErrorName(name) => {
            params::validate_errorname(name)?;
            buf.push(4);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(name, byteorder, buf);
        }
        HeaderField::ReplySerial(rs) => {
            buf.push(5);
            buf.push(1);
            buf.push(b'u');
            buf.push(0);
            pad_to_align(4, buf);
            write_u32(rs.get(), byteorder, buf);
        }
        HeaderField::Destination(dest) => {
            params::validate_busname(dest)?;
            buf.push(6);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(dest, byteorder, buf);
        }
        HeaderField::Sender(snd) => {
            params::validate_busname(snd)?;
            buf.push(7);
            buf.push(1);
            buf.push(b's');
            buf.push(0);
            pad_to_align(4, buf);
            write_string(snd, byteorder, buf);
        }
        HeaderField::Signature(sig) => {
            params::validate_signature(sig)?;
            buf.push(8);
            buf.push(1);
            buf.push(b'g');
            buf.push(0);
            pad_to_align(4, buf);
            write_signature(sig, buf);
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
