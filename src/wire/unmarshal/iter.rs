//! This is a working module to parse a dbus message. It is currently not used in rustbus but it could be in the future. This
//! was more or less a test to see how well/bad this would work out to be.

use crate::params;
use crate::signature;
use crate::wire::unmarshal::base::unmarshal_base;
use crate::wire::unmarshal::Error;
use crate::ByteOrder;

pub struct MessageIter<'a> {
    byteorder: ByteOrder,

    sig: &'a [signature::Type],
    counter: usize,

    source: &'a [u8],
    current_offset: &'a mut usize,
}

impl<'a> MessageIter<'a> {
    pub fn new(
        byteorder: ByteOrder,
        source: &'a [u8],
        start_offset: &'a mut usize,
        sig: &'a [signature::Type],
    ) -> Self {
        MessageIter {
            byteorder,
            source,
            counter: 0,
            current_offset: start_offset,
            sig,
        }
    }

    pub fn next_iter(&'a mut self) -> Option<Result<ParamIter<'a>, Error>> {
        if self.counter >= self.sig.len() {
            None
        } else {
            let iter = ParamIter::new(
                &self.sig[self.counter],
                self.current_offset,
                self.source,
                self.byteorder,
            );
            self.counter += 1;
            iter
        }
    }

    pub fn unmarshal_next<'r, 'buf: 'r, T: crate::wire::unmarshal::traits::Unmarshal<'r, 'buf>>(
        &'buf mut self,
    ) -> Option<Result<T, Error>> {
        if self.counter >= self.sig.len() {
            None
        } else {
            let (bytes, val) = match T::unmarshal(self.byteorder, self.source, *self.current_offset)
            {
                Err(e) => return Some(Err(e)),
                Ok(t) => t,
            };
            *self.current_offset += bytes;
            Some(Ok(val))
        }
    }
}

pub enum ParamIter<'a> {
    Array(ArrayIter<'a>),
    Struct(StructIter<'a>),
    DictEntry(DictEntryIter<'a>),
    Dict(DictIter<'a>),
    Variant(VariantIter<'a>),
    Base(params::Base<'a>),
}

pub struct StructIter<'a> {
    byteorder: ByteOrder,

    sig: &'a [signature::Type],
    counter: usize,

    source: &'a [u8],
    current_offset: &'a mut usize,
}

pub struct DictEntryIter<'a> {
    byteorder: ByteOrder,

    source: &'a [u8],
    current_offset: &'a mut usize,

    key_sig: signature::Base,
    val_sig: &'a signature::Type,

    // first recurse -> key, second recurse -> value
    counter: u8,
}

pub struct VariantIter<'a> {
    byteorder: ByteOrder,

    source: &'a [u8],
    current_offset: &'a mut usize,

    val_sig: signature::Type,

    // first recurse -> value
    counter: u8,
}

pub struct DictIter<'a> {
    byteorder: ByteOrder,

    source: &'a [u8],
    start_offset: usize,
    current_offset: &'a mut usize,
    key_sig: signature::Base,
    val_sig: &'a signature::Type,

    consume_max_bytes: usize,
}

pub struct ArrayIter<'a> {
    byteorder: ByteOrder,

    source: &'a [u8],
    start_offset: usize,
    current_offset: &'a mut usize,
    element_sig: &'a signature::Type,

    consume_max_bytes: usize,
}

impl<'a, 'parent> DictEntryIter<'a> {
    fn recurse(&'parent mut self) -> Option<Result<ParamIter<'parent>, Error>> {
        let iter = if self.counter == 0 {
            // read the key value
            match unmarshal_base(
                self.byteorder,
                self.source,
                self.key_sig,
                *self.current_offset,
            ) {
                Ok((bytes, param)) => {
                    *self.current_offset += bytes;
                    Some(Ok(ParamIter::Base(param)))
                }
                Err(e) => Some(Err(e)),
            }
        } else if self.counter == 1 {
            ParamIter::new(
                &self.val_sig,
                self.current_offset,
                self.source,
                self.byteorder,
            )
        } else {
            None
        };
        if iter.is_some() {
            self.counter += 1;
        }
        iter
    }
}
impl<'a, 'parent> VariantIter<'a> {
    fn recurse(&'parent mut self) -> Option<Result<ParamIter<'parent>, Error>> {
        let iter = if self.counter == 0 {
            ParamIter::new(
                &self.val_sig,
                self.current_offset,
                self.source,
                self.byteorder,
            )
        } else {
            None
        };
        if iter.is_some() {
            self.counter += 1;
        }
        iter
    }
}
impl<'a, 'parent> StructIter<'a> {
    fn recurse(&'parent mut self) -> Option<Result<ParamIter<'parent>, Error>> {
        if self.counter >= self.sig.len() {
            return None;
        }
        let sig = &self.sig[self.counter];
        self.counter += 1;

        ParamIter::new(sig, self.current_offset, self.source, self.byteorder)
    }
}

impl<'a, 'parent> ParamIter<'a> {
    pub fn recurse(&'parent mut self) -> Option<Result<ParamIter<'parent>, Error>> {
        match self {
            ParamIter::Array(array) => array.recurse(),
            ParamIter::DictEntry(de) => de.recurse(),
            ParamIter::Struct(strct) => strct.recurse(),
            ParamIter::Dict(dict) => dict.recurse(),
            ParamIter::Variant(var) => var.recurse(),
            ParamIter::Base(_) => None,
        }
    }

    pub fn is_base(&self) -> bool {
        match self {
            ParamIter::Base(_) => true,
            _ => false,
        }
    }

    pub fn base(self) -> Option<params::Base<'a>> {
        match self {
            ParamIter::Base(b) => Some(b),
            _ => None,
        }
    }

    pub fn new(
        new_sig: &'a signature::Type,
        offset: &'a mut usize,
        source: &'a [u8],
        byteorder: ByteOrder,
    ) -> Option<Result<ParamIter<'a>, Error>> {
        let padding =
            match crate::wire::util::align_offset(new_sig.get_alignment(), source, *offset) {
                Ok(padding) => padding,
                Err(e) => return Some(Err(e)),
            };
        *offset += padding;

        match new_sig {
            signature::Type::Base(b) => match unmarshal_base(byteorder, source, *b, *offset) {
                Ok((bytes, param)) => {
                    *offset += bytes;
                    Some(Ok(ParamIter::Base(param)))
                }
                Err(e) => Some(Err(e)),
            },
            signature::Type::Container(signature::Container::Array(el_sig)) => {
                let item = match make_new_array_iter(offset, source, byteorder, el_sig) {
                    Ok(sub_iter) => Ok(ParamIter::Array(sub_iter)),
                    Err(e) => Err(e),
                };
                Some(item)
            }
            signature::Type::Container(signature::Container::Struct(sig)) => {
                Some(Ok(ParamIter::Struct(StructIter {
                    byteorder,

                    source,
                    current_offset: offset,
                    sig,
                    counter: 0,
                })))
            }
            signature::Type::Container(signature::Container::Dict(key, val)) => {
                let item = match make_new_dict_iter(offset, source, byteorder, *key, val) {
                    Ok(sub_iter) => Ok(ParamIter::Dict(sub_iter)),
                    Err(e) => Err(e),
                };
                Some(item)
            }
            signature::Type::Container(signature::Container::Variant) => {
                let item = match make_new_variant_iter(offset, source, byteorder) {
                    Ok(sub_iter) => Ok(ParamIter::Variant(sub_iter)),
                    Err(e) => Err(e),
                };
                Some(item)
            }
        }
    }
}

impl<'a, 'parent> ArrayIter<'a> {
    fn recurse(&'parent mut self) -> Option<Result<ParamIter<'parent>, Error>> {
        let consumed = *self.current_offset - self.start_offset;
        debug_assert!(consumed <= self.consume_max_bytes);
        if consumed >= self.consume_max_bytes {
            None
        } else {
            ParamIter::new(
                self.element_sig,
                self.current_offset,
                self.source,
                self.byteorder,
            )
        }
    }
}

impl<'a, 'parent> DictIter<'a> {
    fn recurse(&'parent mut self) -> Option<Result<ParamIter<'parent>, Error>> {
        let consumed = *self.current_offset - self.start_offset;
        debug_assert!(consumed <= self.consume_max_bytes);
        if consumed >= self.consume_max_bytes {
            None
        } else {
            Some(Ok(ParamIter::DictEntry(DictEntryIter {
                byteorder: self.byteorder,
                counter: 0,
                source: self.source,
                key_sig: self.key_sig,
                val_sig: self.val_sig,

                current_offset: self.current_offset,
            })))
        }
    }
}

fn make_new_array_iter<'a>(
    offset: &'a mut usize,
    source: &'a [u8],
    byteorder: ByteOrder,
    el_sig: &'a signature::Type,
) -> Result<ArrayIter<'a>, Error> {
    // get child array size
    let (bytes, array_len_bytes) = crate::wire::util::parse_u32(&source[*offset..], byteorder)?;
    debug_assert_eq!(bytes, 4);

    // move offset
    *offset += 4;
    let padding = crate::wire::util::align_offset(el_sig.get_alignment(), source, *offset)?;
    *offset += padding;

    Ok(ArrayIter {
        byteorder,

        source,
        start_offset: *offset,
        current_offset: offset,
        element_sig: el_sig,

        consume_max_bytes: array_len_bytes as usize,
    })
}
fn make_new_variant_iter<'a>(
    offset: &'a mut usize,
    source: &'a [u8],
    byteorder: ByteOrder,
) -> Result<VariantIter<'a>, Error> {
    // get child array size
    let (bytes, sig) = crate::wire::util::unmarshal_signature(&source[*offset..])?;
    debug_assert_eq!(bytes, 4);

    let sig = signature::Type::parse_description(&sig)?.remove(0);

    // move offset
    let padding = crate::wire::util::align_offset(sig.get_alignment(), source, *offset)?;
    *offset += padding;

    Ok(VariantIter {
        byteorder,

        source,
        current_offset: offset,
        val_sig: sig,

        counter: 0,
    })
}
fn make_new_dict_iter<'a>(
    offset: &'a mut usize,
    source: &'a [u8],
    byteorder: ByteOrder,
    key_sig: signature::Base,
    val_sig: &'a signature::Type,
) -> Result<DictIter<'a>, Error> {
    // get child array size
    let (bytes, array_len_bytes) = crate::wire::util::parse_u32(&source[*offset..], byteorder)?;
    debug_assert_eq!(bytes, 4);

    // move offset
    *offset += 4;
    let padding = crate::wire::util::align_offset(8, source, *offset)?;
    *offset += padding;

    Ok(DictIter {
        byteorder,

        source,
        start_offset: *offset,
        current_offset: offset,
        key_sig,
        val_sig,

        consume_max_bytes: array_len_bytes as usize,
    })
}

#[test]
fn test_array_iter() {
    use std::convert::TryFrom;
    let arr = params::Container::try_from(vec![0i32.into(), 1i32.into(), 2i32.into()]).unwrap();

    let mut buf = Vec::new();
    crate::wire::marshal::container::marshal_container_param(
        &arr,
        ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    let mut offset = 0;

    let sig = arr.sig();

    let mut iter = ParamIter::new(&sig, &mut offset, &buf, ByteOrder::LittleEndian)
        .unwrap()
        .unwrap();

    let mut ints = Vec::new();
    while let Some(base) = iter.recurse() {
        if let params::Base::Int32(i) = base.unwrap().base().unwrap() {
            ints.push(i);
        }
    }

    assert_eq!(&[0, 1, 2], ints.as_slice());
}

#[test]
fn test_struct_iter() {
    let s = params::Container::make_struct::<params::Param>(vec![
        0i32.into(),
        "TestTest".into(),
        2i32.into(),
        params::Container::make_struct::<params::Param>(vec![
            1i32.into(),
            "InnerTestTest".into(),
            3i32.into(),
        ])
        .into(),
    ]);

    let mut buf = Vec::new();
    crate::wire::marshal::container::marshal_container_param(&s, ByteOrder::LittleEndian, &mut buf)
        .unwrap();
    let mut offset = 0;

    let sig = s.sig();

    let mut iter = ParamIter::new(&sig, &mut offset, &buf, ByteOrder::LittleEndian)
        .unwrap()
        .unwrap();

    let mut ints = Vec::new();
    let mut strings: Vec<String> = Vec::new();
    while let Some(s) = iter.recurse() {
        let mut sub_iter = s.unwrap();
        if sub_iter.is_base() {
            match &sub_iter.base() {
                Some(params::Base::Int32(i)) => ints.push(*i),
                Some(params::Base::StringRef(sp)) => strings.push(sp.to_string()),
                Some(params::Base::String(s)) => strings.push(s.to_owned()),
                _ => unimplemented!(),
            }
        } else {
            while let Some(base) = sub_iter.recurse() {
                match base.unwrap().base() {
                    Some(params::Base::Int32(i)) => ints.push(i),
                    Some(params::Base::StringRef(sp)) => strings.push(sp.to_owned()),
                    Some(params::Base::String(s)) => strings.push(s),
                    None => {}
                    _ => unimplemented!(),
                }
            }
        }
    }

    assert_eq!(&[0, 2, 1, 3], ints.as_slice());
    assert_eq!(
        &["TestTest".to_owned(), "InnerTestTest".to_owned()],
        strings.as_slice()
    );

    let msg_sig = &[sig];
    offset = 0;
    let mut iter = MessageIter::new(ByteOrder::LittleEndian, &buf, &mut offset, msg_sig);
    let x: (i32, &str, i32, (i32, &str, i32)) = iter.unmarshal_next().unwrap().unwrap();

    assert_eq!(x, (0, "TestTest", 2, (1, "InnerTestTest", 3)));
}
