use crate::message;

pub fn pad_to_align(align_to: usize, buf: &mut Vec<u8>) {
    let padding_needed = align_to - (buf.len() % align_to);
    if padding_needed != align_to {
        buf.resize(buf.len() + padding_needed, 0);
        assert!(buf.len() % align_to == 0);
    }
}

pub fn write_u16(val: u16, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    insert_u16(byteorder, val, &mut buf[pos..]);
}
pub fn write_u32(val: u32, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    insert_u32(byteorder, val, &mut buf[pos..]);
}
pub fn write_u64(val: u64, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let pos = buf.len();
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    buf.push(0);
    insert_u64(byteorder, val, &mut buf[pos..]);
}

pub fn insert_u16(byteorder: message::ByteOrder, val: u16, buf: &mut [u8]) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
        }
        message::ByteOrder::BigEndian => {
            buf[0] = (val >> 8) as u8;
            buf[1] = (val) as u8;
        }
    }
}
pub fn insert_u32(byteorder: message::ByteOrder, val: u32, buf: &mut [u8]) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
            buf[2] = (val >> 16) as u8;
            buf[3] = (val >> 24) as u8;
        }
        message::ByteOrder::BigEndian => {
            buf[0] = (val >> 24) as u8;
            buf[1] = (val >> 16) as u8;
            buf[2] = (val >> 8) as u8;
            buf[3] = (val) as u8;
        }
    }
}
pub fn insert_u64(byteorder: message::ByteOrder, val: u64, buf: &mut [u8]) {
    match byteorder {
        message::ByteOrder::LittleEndian => {
            buf[0] = (val) as u8;
            buf[1] = (val >> 8) as u8;
            buf[2] = (val >> 16) as u8;
            buf[3] = (val >> 24) as u8;
            buf[4] = (val >> 32) as u8;
            buf[5] = (val >> 40) as u8;
            buf[6] = (val >> 48) as u8;
            buf[7] = (val >> 56) as u8;
        }
        message::ByteOrder::BigEndian => {
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

pub fn write_string(val: &str, byteorder: message::ByteOrder, buf: &mut Vec<u8>) {
    let len = val.len() as u32;
    write_u32(len, byteorder, buf);
    buf.extend(val.bytes());
    buf.push(0);
}

pub fn write_signature(val: &str, buf: &mut Vec<u8>) {
    let len = val.len() as u8;
    buf.push(len);
    buf.extend(val.bytes());
    buf.push(0);
}
