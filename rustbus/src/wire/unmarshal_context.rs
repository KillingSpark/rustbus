use crate::ByteOrder;

use super::{
    errors::UnmarshalError,
    unmarshal::UnmarshalResult,
    util::{parse_u16, parse_u32, parse_u64, unmarshal_signature, unmarshal_str},
};

pub struct UnmarshalContext<'fds, 'buf> {
    pub fds: &'fds [crate::wire::UnixFd],
    pub byteorder: ByteOrder,
    buf: &'buf [u8],
    offset: usize,
}

impl<'fds, 'buf> UnmarshalContext<'fds, 'buf> {
    pub fn new(
        fds: &'fds [crate::wire::UnixFd],
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> UnmarshalContext<'fds, 'buf> {
        Self {
            fds,
            byteorder,
            buf,
            offset,
        }
    }

    pub fn align_to(&mut self, alignment: usize) -> Result<usize, UnmarshalError> {
        let padding = crate::wire::util::align_offset(alignment, self.buf, self.offset)?;

        if self.offset + padding > self.buf.len() {
            Err(UnmarshalError::NotEnoughBytes)
        } else {
            self.offset += padding;
            Ok(padding)
        }
    }

    pub fn remainder(&self) -> &[u8] {
        &self.buf[self.offset..]
    }

    pub fn read_u8(&mut self) -> UnmarshalResult<u8> {
        if self.remainder().is_empty() {
            Err(UnmarshalError::NotEnoughBytes)
        } else {
            self.offset += 1;
            Ok((1, self.buf[self.offset - 1]))
        }
    }

    pub fn read_i16(&mut self) -> UnmarshalResult<i16> {
        self.read_u16()
            .map(|(consumed, value)| (consumed, value as i16))
    }

    pub fn read_u16(&mut self) -> UnmarshalResult<u16> {
        self.align_to(2)?;
        let (consumed_value, value) = parse_u16(self.remainder(), self.byteorder)?;
        self.offset += consumed_value;
        Ok((consumed_value, value))
    }

    pub fn read_i32(&mut self) -> UnmarshalResult<i32> {
        self.read_u32()
            .map(|(consumed, value)| (consumed, value as i32))
    }

    pub fn read_u32(&mut self) -> UnmarshalResult<u32> {
        self.align_to(4)?;
        let (consumed_value, value) = parse_u32(self.remainder(), self.byteorder)?;
        self.offset += consumed_value;
        Ok((consumed_value, value))
    }

    pub fn read_i64(&mut self) -> UnmarshalResult<i64> {
        self.read_u64()
            .map(|(consumed, value)| (consumed, value as i64))
    }

    pub fn read_u64(&mut self) -> UnmarshalResult<u64> {
        self.align_to(8)?;
        let (consumed_value, value) = parse_u64(self.remainder(), self.byteorder)?;
        self.offset += consumed_value;
        Ok((consumed_value, value))
    }

    pub fn read_str(&mut self) -> UnmarshalResult<&'buf str> {
        self.align_to(4)?;
        let (consumed_value, value) = unmarshal_str(self.byteorder, &self.buf[self.offset..])?;
        self.offset += consumed_value;
        Ok((consumed_value, value))
    }

    pub fn read_signature(&mut self) -> UnmarshalResult<&'buf str> {
        let (consumed_value, value) = unmarshal_signature(&self.buf[self.offset..])?;
        self.offset += consumed_value;
        Ok((consumed_value, value))
    }

    pub fn read_u8_slice(&mut self) -> UnmarshalResult<&'buf [u8]> {
        let (bytes_in_len, bytes_in_array) = self.read_u32()?;

        let (_, elements) = self.read_raw(bytes_in_array as usize)?;

        let total_bytes_used = bytes_in_len + bytes_in_array as usize;
        Ok((total_bytes_used, elements))
    }

    pub fn read_raw(&mut self, length: usize) -> UnmarshalResult<&'buf [u8]> {
        if length as usize > self.remainder().len() {
            return Err(UnmarshalError::NotEnoughBytes);
        }

        let elements = &&self.buf[self.offset..][..length as usize];
        self.offset += length;

        Ok((length, elements))
    }

    pub fn reset(&mut self, reset_by: usize) {
        self.offset -= reset_by;
    }
}
