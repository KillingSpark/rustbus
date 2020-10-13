//! Utility functions used often in many places

use crate::wire::unmarshal;
use crate::wire::unmarshal::UnmarshalResult;
use crate::ByteOrder;

pub fn pad_to_align(align_to: usize, buf: &mut Vec<u8>) {
    let padding_needed = align_to - (buf.len() % align_to);
    if padding_needed != align_to {
        buf.resize(buf.len() + padding_needed, 0);
        assert!(buf.len() % align_to == 0);
    }
}

pub fn write_u16(val: u16, byteorder: ByteOrder, buf: &mut Vec<u8>) {
    match byteorder {
        ByteOrder::LittleEndian => buf.extend(&val.to_le_bytes()[..]),
        ByteOrder::BigEndian => buf.extend(&val.to_be_bytes()[..]),
    }
}
pub fn write_u32(val: u32, byteorder: ByteOrder, buf: &mut Vec<u8>) {
    match byteorder {
        ByteOrder::LittleEndian => buf.extend(&val.to_le_bytes()[..]),
        ByteOrder::BigEndian => buf.extend(&val.to_be_bytes()[..]),
    }
}
pub fn write_u64(val: u64, byteorder: ByteOrder, buf: &mut Vec<u8>) {
    match byteorder {
        ByteOrder::LittleEndian => buf.extend(&val.to_le_bytes()[..]),
        ByteOrder::BigEndian => buf.extend(&val.to_be_bytes()[..]),
    }
}

pub fn insert_u16(byteorder: ByteOrder, val: u16, buf: &mut [u8]) {
    match byteorder {
        ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
        }
        ByteOrder::BigEndian => {
            buf[0] = (val >> 8) as u8;
            buf[1] = (val) as u8;
        }
    }
}
pub fn insert_u32(byteorder: ByteOrder, val: u32, buf: &mut [u8]) {
    match byteorder {
        ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
            buf[2] = (val >> 16) as u8;
            buf[3] = (val >> 24) as u8;
        }
        ByteOrder::BigEndian => {
            buf[0] = (val >> 24) as u8;
            buf[1] = (val >> 16) as u8;
            buf[2] = (val >> 8) as u8;
            buf[3] = (val) as u8;
        }
    }
}
pub fn insert_u64(byteorder: ByteOrder, val: u64, buf: &mut [u8]) {
    match byteorder {
        ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
            buf[2] = (val >> 16) as u8;
            buf[3] = (val >> 24) as u8;
            buf[4] = (val >> 32) as u8;
            buf[5] = (val >> 40) as u8;
            buf[6] = (val >> 48) as u8;
            buf[7] = (val >> 56) as u8;
        }
        ByteOrder::BigEndian => {
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

fn extend_with_memcopy(val: &str, buf: &mut Vec<u8>) {
    buf.reserve(val.len() + 1);
    unsafe {
        let target = buf.as_mut_ptr().add(buf.len());
        std::ptr::copy(val.as_ptr(), target, val.len());
        buf.set_len(buf.len() + val.len());
    }
}

pub fn write_string(val: &str, byteorder: ByteOrder, buf: &mut Vec<u8>) {
    let len = val.len() as u32;
    write_u32(len, byteorder, buf);
    extend_with_memcopy(val, buf);
    buf.push(0);
}

pub fn write_signature(val: &str, buf: &mut Vec<u8>) {
    let len = val.len() as u8;
    buf.push(len);
    extend_with_memcopy(val, buf);
    buf.push(0);
}

pub fn parse_u64(number: &[u8], byteorder: ByteOrder) -> UnmarshalResult<u64> {
    if number.len() < 8 {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    let val = match byteorder {
        ByteOrder::LittleEndian => {
            (number[0] as u64)
                + ((number[1] as u64) << 8)
                + ((number[2] as u64) << 16)
                + ((number[3] as u64) << 24)
                + ((number[4] as u64) << 32)
                + ((number[5] as u64) << 40)
                + ((number[6] as u64) << 48)
                + ((number[7] as u64) << 56)
        }
        ByteOrder::BigEndian => {
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

pub fn parse_u32(number: &[u8], byteorder: ByteOrder) -> UnmarshalResult<u32> {
    if number.len() < 4 {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    let val = match byteorder {
        ByteOrder::LittleEndian => {
            (number[0] as u32)
                + ((number[1] as u32) << 8)
                + ((number[2] as u32) << 16)
                + ((number[3] as u32) << 24)
        }
        ByteOrder::BigEndian => {
            (number[3] as u32)
                + ((number[2] as u32) << 8)
                + ((number[1] as u32) << 16)
                + ((number[0] as u32) << 24)
        }
    };
    Ok((4, val))
}

pub fn parse_u16(number: &[u8], byteorder: ByteOrder) -> UnmarshalResult<u16> {
    if number.len() < 2 {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    let val = match byteorder {
        ByteOrder::LittleEndian => (number[0] as u16) + ((number[1] as u16) << 8),
        ByteOrder::BigEndian => (number[1] as u16) + ((number[0] as u16) << 8),
    };
    Ok((2, val))
}

pub fn align_offset(align_to: usize, buf: &[u8], offset: usize) -> Result<usize, unmarshal::Error> {
    let padding_delete = align_to - (offset % align_to);
    let padding_delete = if padding_delete == align_to {
        0
    } else {
        padding_delete
    };

    if buf[offset..].len() < padding_delete {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    for x in 0..padding_delete {
        if buf[offset + x] != b'\0' {
            return Err(unmarshal::Error::PaddingContainedData);
        }
    }
    Ok(padding_delete)
}

pub fn unmarshal_signature(buf: &[u8]) -> UnmarshalResult<&str> {
    if buf.is_empty() {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    let len = buf[0] as usize;
    if buf.len() < len + 2 {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    let sig_buf = &buf[1..];
    let string = std::str::from_utf8(&sig_buf[..len])
        .map_err(|_| crate::params::validation::Error::InvalidUtf8)?;
    Ok((len + 2, string))
}

pub fn unmarshal_string(byteorder: ByteOrder, buf: &[u8]) -> UnmarshalResult<String> {
    let (bytes, string) = unmarshal_str(byteorder, buf)?;
    Ok((bytes, string.into()))
}

pub fn unmarshal_str<'r, 'a: 'r>(byteorder: ByteOrder, buf: &'a [u8]) -> UnmarshalResult<&'r str> {
    let len = parse_u32(buf, byteorder)?.1 as usize;
    if buf.len() < len + 5 {
        return Err(unmarshal::Error::NotEnoughBytes);
    }
    let str_buf = &buf[4..];
    let string = std::str::from_utf8(&str_buf[..len])
        .map_err(|_| crate::params::validation::Error::InvalidUtf8)?;
    if string.contains('\0') {
        return Err(crate::params::validation::Error::StringContainsNullByte.into());
    }
    Ok((len + 5, string))
}
