use crate::message::Error;
use crate::message::Result;
use crate::params::*;
use crate::signature;

/// The Types a message can have as parameters
/// There are From<T> impls for most of the Base ones
///
/// 'a is the lifetime of the Container, 'e the liftime of the params which may be longer
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Param<'a, 'e> {
    Base(Base<'a>),
    Container(Container<'a, 'e>),
}

/// The base types a message can have as parameters
/// There are From<T> impls for most of them
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Base<'a> {
    // Owned
    Double(u64),
    Byte(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    UnixFd(u32),
    Int64(i64),
    Uint64(u64),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),

    // By ref
    DoubleRef(&'a u64),
    ByteRef(&'a u8),
    Int16Ref(&'a i16),
    Uint16Ref(&'a u16),
    Int32Ref(&'a i32),
    Uint32Ref(&'a u32),
    UnixFdRef(&'a u32),
    Int64Ref(&'a i64),
    Uint64Ref(&'a u64),
    StringRef(&'a str),
    SignatureRef(&'a str),
    ObjectPathRef(&'a str),
    BooleanRef(&'a bool),
}

pub type DictMap<'a, 'e> = std::collections::HashMap<Base<'a>, Param<'a, 'e>>;

/// The container types a message can have as parameters
///
/// 'a is the lifetime of the Container, 'e the liftime of the params which may be longer
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Container<'e, 'a: 'e> {
    // Owned
    Array(Array<'e, 'a>),
    Struct(Vec<Param<'a, 'e>>),
    Dict(Dict<'a, 'e>),
    Variant(Box<Variant<'a, 'e>>),
    // By ref
    ArrayRef(ArrayRef<'a, 'e>),
    StructRef(&'a [Param<'a, 'e>]),
    DictRef(DictRef<'a, 'e>),
}

impl<'e, 'a: 'e> Container<'a, 'e> {
    pub fn make_struct<P: Into<Param<'a, 'e>>>(elements: Vec<P>) -> Container<'a, 'e> {
        Container::Struct(elements.into_iter().map(std::convert::Into::into).collect())
    }

    pub fn make_variant<P: Into<Param<'a, 'e>>>(element: P) -> Container<'a, 'e> {
        let param: Param = element.into();

        Container::Variant(Box::new(Variant {
            sig: param.sig(),
            value: param,
        }))
    }

    pub fn make_array_ref(
        element_sig: &str,
        elements: &'a [Param<'a, 'e>],
    ) -> Result<Container<'a, 'e>> {
        let mut sigs =
            signature::Type::parse_description(element_sig).map_err(Error::InvalidSignature)?;

        if sigs.len() != 1 {
            return Err(Error::InvalidSignatureTooManyTypes);
        }

        let sig = sigs.remove(0);
        Self::make_array_ref_with_sig(sig, elements)
    }

    pub fn make_array_ref_with_sig(
        element_sig: signature::Type,
        elements: &'a [Param<'a, 'e>],
    ) -> Result<Container<'a, 'e>> {
        let arr: ArrayRef<'a, 'e> = ArrayRef {
            element_sig,
            values: elements,
        };

        validate_array(&arr.values, &arr.element_sig)?;

        Ok(Container::ArrayRef(arr))
    }

    pub fn make_array<P: Into<Param<'a, 'e>>>(
        element_sig: &str,
        elements: Vec<P>,
    ) -> Result<Container<'a, 'e>> {
        let mut sigs =
            signature::Type::parse_description(element_sig).map_err(Error::InvalidSignature)?;

        if sigs.len() != 1 {
            return Err(Error::InvalidSignatureTooManyTypes);
        }

        let sig = sigs.remove(0);
        Self::make_array_with_sig(sig, elements)
    }

    pub fn make_array_with_sig<P: Into<Param<'a, 'e>>>(
        element_sig: signature::Type,
        elements: Vec<P>,
    ) -> Result<Container<'a, 'e>> {
        let arr: Array<'a, 'e> = Array {
            element_sig,
            values: elements.into_iter().map(std::convert::Into::into).collect(),
        };

        validate_array(&arr.values, &arr.element_sig)?;

        Ok(Container::Array(arr))
    }

    pub fn make_dict<K: Into<Base<'e>>, V: Into<Param<'a, 'e>>>(
        key_sig: &str,
        val_sig: &str,
        map: std::collections::HashMap<K, V>,
    ) -> Result<Container<'a, 'e>> {
        let mut valsigs =
            signature::Type::parse_description(val_sig).map_err(Error::InvalidSignature)?;

        if valsigs.len() != 1 {
            return Err(Error::InvalidSignatureTooManyTypes);
        }

        let value_sig = valsigs.remove(0);
        let mut keysigs =
            signature::Type::parse_description(key_sig).map_err(Error::InvalidSignature)?;

        if keysigs.len() != 1 {
            return Err(Error::InvalidSignatureTooManyTypes);
        }
        let key_sig = keysigs.remove(0);
        let key_sig = if let signature::Type::Base(sig) = key_sig {
            sig
        } else {
            return Err(Error::InvalidSignatureShouldBeBase);
        };

        Self::make_dict_with_sig(key_sig, value_sig, map)
    }

    pub fn make_dict_with_sig<K: Into<Base<'e>>, V: Into<Param<'a, 'e>>>(
        key_sig: signature::Base,
        value_sig: signature::Type,
        map: std::collections::HashMap<K, V>,
    ) -> Result<Container<'a, 'e>> {
        let dict = Dict {
            key_sig,
            value_sig,
            map: map.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
        };

        validate_dict(&dict.map, dict.key_sig, &dict.value_sig)?;

        Ok(Container::Dict(dict))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variant<'a, 'e: 'a> {
    pub sig: signature::Type,
    pub value: Param<'a, 'e>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Array<'a, 'e: 'a> {
    pub element_sig: signature::Type,
    pub values: Vec<Param<'a, 'e>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ArrayRef<'a, 'e: 'a> {
    pub element_sig: signature::Type,
    pub values: &'a [Param<'a, 'e>],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dict<'a, 'e: 'a> {
    pub key_sig: signature::Base,
    pub value_sig: signature::Type,
    pub map: DictMap<'a, 'e>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DictRef<'a, 'e: 'a> {
    pub key_sig: signature::Base,
    pub value_sig: signature::Type,
    pub map: &'a DictMap<'a, 'e>,
}

impl<'a, 'e> Param<'a, 'e> {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Param::Base(b) => b.make_signature(buf),
            Param::Container(c) => c.make_signature(buf),
        }
    }
    pub fn sig(&self) -> signature::Type {
        match self {
            Param::Base(b) => b.sig(),
            Param::Container(c) => c.sig(),
        }
    }
}

impl<'a> Base<'a> {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Base::Boolean(_) => buf.push('b'),
            Base::Double(_) => buf.push('d'),
            Base::Byte(_) => buf.push('y'),
            Base::Int16(_) => buf.push('n'),
            Base::Uint16(_) => buf.push('q'),
            Base::Int32(_) => buf.push('i'),
            Base::Uint32(_) => buf.push('u'),
            Base::UnixFd(_) => buf.push('h'),
            Base::Int64(_) => buf.push('x'),
            Base::Uint64(_) => buf.push('t'),
            Base::ObjectPath(_) => buf.push('o'),
            Base::String(_) => buf.push('s'),
            Base::Signature(_) => buf.push('g'),
            Base::BooleanRef(_) => buf.push('b'),
            Base::DoubleRef(_) => buf.push('d'),
            Base::ByteRef(_) => buf.push('y'),
            Base::Int16Ref(_) => buf.push('n'),
            Base::Uint16Ref(_) => buf.push('q'),
            Base::Int32Ref(_) => buf.push('i'),
            Base::Uint32Ref(_) => buf.push('u'),
            Base::UnixFdRef(_) => buf.push('h'),
            Base::Int64Ref(_) => buf.push('x'),
            Base::Uint64Ref(_) => buf.push('t'),
            Base::ObjectPathRef(_) => buf.push('o'),
            Base::StringRef(_) => buf.push('s'),
            Base::SignatureRef(_) => buf.push('g'),
        }
    }

    pub fn sig(&self) -> signature::Type {
        let sig: signature::Base = self.into();
        signature::Type::Base(sig)
    }
}
impl<'a, 'e> Container<'a, 'e> {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Container::Array(elements) => {
                buf.push('a');
                elements.element_sig.to_str(buf);
            }
            Container::Dict(map) => {
                buf.push('a');
                buf.push('{');
                map.key_sig.to_str(buf);
                map.value_sig.to_str(buf);
                buf.push('}');
            }
            Container::Struct(elements) => {
                buf.push('(');
                for el in elements {
                    el.make_signature(buf);
                }
                buf.push(')');
            }
            Container::Variant(_) => {
                buf.push('v');
            }
            Container::ArrayRef(elements) => {
                buf.push('a');
                elements.element_sig.to_str(buf);
            }
            Container::DictRef(map) => {
                buf.push('a');
                buf.push('{');
                map.key_sig.to_str(buf);
                map.value_sig.to_str(buf);
                buf.push('}');
            }
            Container::StructRef(elements) => {
                buf.push('(');
                for el in *elements {
                    el.make_signature(buf);
                }
                buf.push(')');
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Container::Array(elements) => elements.values.len(),
            Container::ArrayRef(elements) => elements.values.len(),
            Container::Dict(map) => map.map.len(),
            Container::DictRef(map) => map.map.len(),
            Container::Struct(elements) => elements.len(),
            Container::StructRef(elements) => elements.len(),
            Container::Variant(_) => 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn sig(&self) -> signature::Type {
        let sig: signature::Container = self.into();
        signature::Type::Container(sig)
    }
}
