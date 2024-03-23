//! All things relevant to marshalling content into raw bytes
//!
//! * `base` and `container` are for the Param approach that map dbus concepts to enums/structs
//! * `traits` is for the trait based approach

use std::num::NonZeroU32;

use crate::message_builder;
use crate::params;
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

    if let Some(serial) = msg.dynheader.response_serial {
        marshal_header_reply_serial(byteorder, serial, buf)?;
    }
    if let Some(int) = &msg.dynheader.interface {
        marshal_header_interface(byteorder, int, buf)?;
    }
    if let Some(dest) = &msg.dynheader.destination {
        marshal_header_destination(byteorder, dest, buf)?;
    }
    if let Some(sender) = &msg.dynheader.sender {
        marshal_header_sender(byteorder, sender, buf)?;
    }
    if let Some(mem) = &msg.dynheader.member {
        marshal_header_member(byteorder, mem, buf)?;
    }
    if let Some(obj) = &msg.dynheader.object {
        marshal_header_path(byteorder, obj, buf)?;
    }
    if let Some(err_name) = &msg.dynheader.error_name {
        marshal_header_errorname(byteorder, err_name, buf)?;
    }
    if !msg.get_buf().is_empty() {
        marshal_header_signature(msg.get_sig(), buf)?;
    }
    if !msg.body.get_fds().is_empty() {
        marshal_header_unix_fds(byteorder, msg.body.get_fds().len() as u32, buf)?;
    }
    let len = buf.len() - pos - 4; // -4 the bytes for the length indicator do not count
    insert_u32(byteorder, len as u32, &mut buf[pos..pos + 4]);

    Ok(())
}

fn marshal_header_field(field_no: u8, sig: &str, buf: &mut Vec<u8>) {
    pad_to_align(8, buf);
    buf.push(field_no);
    buf.push(sig.len() as u8);
    buf.extend_from_slice(sig.as_bytes());
    buf.push(0);
    pad_to_align(4, buf);
}

fn marshal_header_path(byteorder: ByteOrder, path: &str, buf: &mut Vec<u8>) -> MarshalResult<()> {
    params::validate_object_path(path)?;
    marshal_header_field(1, "o", buf);
    write_string(path, byteorder, buf);
    Ok(())
}

fn marshal_header_interface(
    byteorder: ByteOrder,
    interface: &str,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    params::validate_interface(interface)?;
    marshal_header_field(2, "s", buf);
    write_string(interface, byteorder, buf);
    Ok(())
}

fn marshal_header_member(
    byteorder: ByteOrder,
    member: &str,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    params::validate_membername(member)?;
    marshal_header_field(3, "s", buf);
    write_string(member, byteorder, buf);
    Ok(())
}

fn marshal_header_errorname(
    byteorder: ByteOrder,
    error: &str,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    params::validate_errorname(error)?;
    marshal_header_field(4, "s", buf);
    write_string(error, byteorder, buf);
    Ok(())
}

fn marshal_header_reply_serial(
    byteorder: ByteOrder,
    serial: NonZeroU32,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    marshal_header_field(5, "u", buf);
    write_u32(serial.get(), byteorder, buf);
    Ok(())
}

fn marshal_header_destination(
    byteorder: ByteOrder,
    destination: &str,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    params::validate_busname(destination)?;
    marshal_header_field(6, "s", buf);
    write_string(destination, byteorder, buf);
    Ok(())
}

fn marshal_header_sender(
    byteorder: ByteOrder,
    sender: &str,
    buf: &mut Vec<u8>,
) -> MarshalResult<()> {
    params::validate_busname(sender)?;
    marshal_header_field(7, "s", buf);
    write_string(sender, byteorder, buf);
    Ok(())
}

fn marshal_header_signature(signature: &str, buf: &mut Vec<u8>) -> MarshalResult<()> {
    params::validate_signature(signature)?;
    marshal_header_field(8, "g", buf);
    write_signature(signature, buf);
    Ok(())
}

fn marshal_header_unix_fds(byteorder: ByteOrder, fds: u32, buf: &mut Vec<u8>) -> MarshalResult<()> {
    marshal_header_field(9, "u", buf);
    write_u32(fds, byteorder, buf);
    Ok(())
}
