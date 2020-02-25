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
    ArrayRef(&'a Array<'a, 'e>),
    StructRef(&'a Vec<Param<'a, 'e>>),
    DictRef(&'a Dict<'a, 'e>),
    VariantRef(&'a Variant<'a, 'e>),
}

impl<'e, 'a: 'e> Container<'a, 'e> {
    pub fn make_array(
        element_sig: &str,
        elements: Vec<Param<'a, 'e>>,
    ) -> Result<Container<'a, 'e>> {
        let mut sigs =
            signature::Type::parse_description(element_sig).map_err(Error::InvalidSignature)?;

        if sigs.len() != 1 {
            return Err(Error::InvalidSignatureTooManyTypes);
        }

        let sig = sigs.remove(0);
        Self::make_array_with_sig(sig, elements)
    }

    pub fn make_array_with_sig(
        element_sig: signature::Type,
        elements: Vec<Param<'a, 'e>>,
    ) -> Result<Container<'a, 'e>> {
        let arr: Array<'a, 'e> = Array {
            element_sig,
            values: elements,
        };

        validate_array(&arr)?;

        Ok(Container::Array(arr))
    }

    pub fn make_dict(
        key_sig: &str,
        val_sig: &str,
        map: DictMap<'a, 'e>,
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

    pub fn make_dict_with_sig(
        key_sig: signature::Base,
        value_sig: signature::Type,
        map: DictMap<'a, 'e>,
    ) -> Result<Container<'a, 'e>> {
        let dict = Dict {
            key_sig,
            value_sig,
            map,
        };

        validate_dict(&dict)?;

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
pub struct Dict<'a, 'e: 'a> {
    pub key_sig: signature::Base,
    pub value_sig: signature::Type,
    pub map: DictMap<'a, 'e>,
}
