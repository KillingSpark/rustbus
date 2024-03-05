use crate::ByteOrder;

use super::{
    errors::UnmarshalError,
    unmarshal::UnmarshalResult,
    util::{parse_u16, parse_u32, parse_u64, unmarshal_signature, unmarshal_str},
    UnixFd,
};

#[derive(Debug, Clone, Copy)]
pub struct UnmarshalContext<'fds, 'buf> {
    pub byteorder: ByteOrder,
    fds: &'fds [crate::wire::UnixFd],
    cursor: Cursor<'buf>,
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
            cursor: Cursor { buf, offset },
        }
    }

    pub fn sub_context(&mut self, length: usize) -> UnmarshalResult<UnmarshalContext<'fds, 'buf>> {
        let region = self.read_raw(length)?;
        Ok(UnmarshalContext::new(self.fds, self.byteorder, region, 0))
    }

    pub fn align_to(&mut self, alignment: usize) -> Result<usize, UnmarshalError> {
        self.cursor.align_to(alignment)
    }

    pub fn remainder(&self) -> &[u8] {
        self.cursor.remainder()
    }

    pub fn read_u8(&mut self) -> UnmarshalResult<u8> {
        self.cursor.read_u8()
    }

    pub fn read_i16(&mut self) -> UnmarshalResult<i16> {
        self.cursor.read_i16(self.byteorder)
    }

    pub fn read_u16(&mut self) -> UnmarshalResult<u16> {
        self.cursor.read_u16(self.byteorder)
    }

    pub fn read_i32(&mut self) -> UnmarshalResult<i32> {
        self.cursor.read_i32(self.byteorder)
    }

    pub fn read_u32(&mut self) -> UnmarshalResult<u32> {
        self.cursor.read_u32(self.byteorder)
    }

    pub fn read_unixfd(&mut self) -> UnmarshalResult<UnixFd> {
        let idx = self.cursor.read_u32(self.byteorder)?;
        if self.fds.len() <= idx as usize {
            Err(UnmarshalError::BadFdIndex(idx as usize))
        } else {
            let val = &self.fds[idx as usize];
            Ok(val.clone())
        }
    }

    pub fn read_i64(&mut self) -> UnmarshalResult<i64> {
        self.cursor.read_i64(self.byteorder)
    }

    pub fn read_u64(&mut self) -> UnmarshalResult<u64> {
        self.cursor.read_u64(self.byteorder)
    }

    pub fn read_str(&mut self) -> UnmarshalResult<&'buf str> {
        self.cursor.read_str(self.byteorder)
    }

    pub fn read_signature(&mut self) -> UnmarshalResult<&'buf str> {
        self.cursor.read_signature()
    }

    pub fn read_u8_slice(&mut self) -> UnmarshalResult<&'buf [u8]> {
        self.cursor.read_u8_slice(self.byteorder)
    }

    pub fn read_raw(&mut self, length: usize) -> UnmarshalResult<&'buf [u8]> {
        self.cursor.read_raw(length)
    }

    pub fn reset(&mut self, reset_by: usize) {
        self.cursor.reset(reset_by)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'buf> Cursor<'buf> {
    pub fn new(buf: &[u8]) -> Cursor {
        Cursor { buf, offset: 0 }
    }

    pub fn consumed(&self) -> usize {
        self.offset
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
            Ok(self.buf[self.offset - 1])
        }
    }

    pub fn read_i16(&mut self, byteorder: ByteOrder) -> UnmarshalResult<i16> {
        self.read_u16(byteorder).map(|value| value as i16)
    }

    pub fn read_u16(&mut self, byteorder: ByteOrder) -> UnmarshalResult<u16> {
        self.align_to(2)?;
        let value = parse_u16(self.remainder(), byteorder)?;
        self.offset += 2;
        Ok(value)
    }

    pub fn read_i32(&mut self, byteorder: ByteOrder) -> UnmarshalResult<i32> {
        self.read_u32(byteorder).map(|value| value as i32)
    }

    pub fn read_u32(&mut self, byteorder: ByteOrder) -> UnmarshalResult<u32> {
        self.align_to(4)?;
        let value = parse_u32(self.remainder(), byteorder)?;
        self.offset += 4;
        Ok(value)
    }

    pub fn read_i64(&mut self, byteorder: ByteOrder) -> UnmarshalResult<i64> {
        self.read_u64(byteorder).map(|value| value as i64)
    }

    pub fn read_u64(&mut self, byteorder: ByteOrder) -> UnmarshalResult<u64> {
        self.align_to(8)?;
        let value = parse_u64(self.remainder(), byteorder)?;
        self.offset += 8;
        Ok(value)
    }

    pub fn read_str(&mut self, byteorder: ByteOrder) -> UnmarshalResult<&'buf str> {
        self.align_to(4)?;
        let (bytes, value) = unmarshal_str(byteorder, &self.buf[self.offset..])?;
        self.offset += bytes;
        Ok(value)
    }

    pub fn read_signature(&mut self) -> UnmarshalResult<&'buf str> {
        let (bytes, value) = unmarshal_signature(&self.buf[self.offset..])?;
        self.offset += bytes;
        Ok(value)
    }

    pub fn read_u8_slice(&mut self, byteorder: ByteOrder) -> UnmarshalResult<&'buf [u8]> {
        self.align_to(4)?;
        let bytes_in_array = self.read_u32(byteorder)?;

        let elements = self.read_raw(bytes_in_array as usize)?;

        Ok(elements)
    }

    pub fn read_raw(&mut self, length: usize) -> UnmarshalResult<&'buf [u8]> {
        if length > self.remainder().len() {
            return Err(UnmarshalError::NotEnoughBytes);
        }

        let elements = &&self.buf[self.offset..][..length];
        self.offset += length;

        Ok(elements)
    }

    pub fn reset(&mut self, reset_by: usize) {
        self.offset -= reset_by;
    }

    pub fn advance(&mut self, advance_by: usize) {
        self.offset += advance_by;
    }
}
