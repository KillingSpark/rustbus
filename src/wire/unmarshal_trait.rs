//! An unfinished first try at implementing an unmarshal trait. To apply this to message decoding signature checking would have to be done in some way...

use crate::message::ByteOrder;
use crate::wire::unmarshal;
use crate::wire::util;

pub trait Unmarshal<'r, 'buf: 'r>: Sized {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self>;

    fn alignment() -> usize;
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for u64 {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        util::parse_u64(&buf[offset..], byteorder)
    }
    fn alignment() -> usize {
        8
    }
}

impl<'r, 'buf: 'r> Unmarshal<'r, 'buf> for &'r str {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        util::unmarshal_str(byteorder, &buf[offset..])
    }
    fn alignment() -> usize {
        4
    }
}

impl<'r, 'buf: 'r, E: Unmarshal<'r, 'buf>> Unmarshal<'r, 'buf> for Vec<E> {
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(4, buf, offset)?;
        let offset = offset + padding;
        let (_, bytes_in_array) = util::parse_u32(&buf[offset..], byteorder)?;
        let offset = offset + 4;

        let first_elem_padding = util::align_offset(E::alignment(), buf, offset)?;
        let offset = offset + first_elem_padding;

        let mut elements = Vec::new();
        let mut bytes_used_counter = 0;
        eprintln!("bytes used {}", bytes_used_counter);
        while bytes_used_counter < bytes_in_array as usize {
            let elem_padding =
                util::align_offset(E::alignment(), buf, offset + bytes_used_counter)?;
            bytes_used_counter += elem_padding;
            let (bytes_used, element) = E::unmarshal(byteorder, buf, offset + bytes_used_counter)?;
            elements.push(element);
            bytes_used_counter += bytes_used;
            eprintln!("bytes used {}", bytes_used_counter);
        }

        let total_bytes_used = padding + 4 + first_elem_padding + bytes_used_counter;

        Ok((total_bytes_used, elements))
    }
    fn alignment() -> usize {
        4
    }
}

impl<'r, 'buf: 'r, K: Unmarshal<'r, 'buf> + std::hash::Hash + Eq, V: Unmarshal<'r, 'buf>>
    Unmarshal<'r, 'buf> for std::collections::HashMap<K, V>
{
    fn unmarshal(
        byteorder: ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let padding = util::align_offset(4, buf, offset)?;
        let offset = offset + padding;
        let (_, bytes_in_array) = util::parse_u32(&buf[offset..], byteorder)?;
        let offset = offset + 4;

        let first_elem_padding = util::align_offset(8, buf, offset)?;
        let offset = offset + first_elem_padding;

        let mut map = std::collections::HashMap::new();
        let mut bytes_used_counter = 0;
        eprintln!("bytes used {}", bytes_used_counter);
        while bytes_used_counter < bytes_in_array as usize {
            let elem_padding = util::align_offset(8, buf, offset + bytes_used_counter)?;
            bytes_used_counter += elem_padding;

            let (key_bytes_used, key) = K::unmarshal(byteorder, buf, offset + bytes_used_counter)?;
            bytes_used_counter += key_bytes_used;

            let val_padding = util::align_offset(V::alignment(), buf, offset + bytes_used_counter)?;
            bytes_used_counter += val_padding;

            let (val_bytes_used, val) = V::unmarshal(byteorder, buf, offset + bytes_used_counter)?;
            bytes_used_counter += val_bytes_used;

            map.insert(key, val);
            eprintln!("bytes used {}", bytes_used_counter);
        }

        let total_bytes_used = padding + 4 + first_elem_padding + bytes_used_counter;

        Ok((total_bytes_used, map))
    }
    fn alignment() -> usize {
        4
    }
}

#[test]
fn test_unmarshal_trait() {
    use crate::Marshal;

    let mut buf = Vec::new();
    let original = &["a", "b"];
    original.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();

    let (_, v) = Vec::<&str>::unmarshal(ByteOrder::LittleEndian, &buf, 0).unwrap();

    assert_eq!(original, v.as_slice());

    buf.clear();

    let mut original = std::collections::HashMap::new();
    original.insert(0u64, "abc");
    original.insert(1u64, "dce");
    original.insert(2u64, "fgh");

    original.marshal(ByteOrder::LittleEndian, &mut buf).unwrap();

    let (_, map) =
        std::collections::HashMap::<u64, &str>::unmarshal(ByteOrder::LittleEndian, &buf, 0)
            .unwrap();
    assert_eq!(original, map);
}
