use crate::message::Error;
use crate::message::Result;
use crate::params::*;
use crate::signature;

/// The base types a message can have as parameters
/// There are From<T> impls for most of them
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Base {
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
}

pub type DictMap = std::collections::HashMap<Base, Param>;

/// The container types a message can have as parameters
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Container {
    Array(Array),
    Struct(Vec<Param>),
    Dict(Dict),
    Variant(Box<Variant>),
}

impl Container {
    pub fn make_array(element_sig: &str, elements: Vec<Param>) -> Result<Container> {
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
        elements: Vec<Param>,
    ) -> Result<Container> {
        let arr = Array {
            element_sig,
            values: elements,
        };

        validate_array(&arr)?;

        Ok(Container::Array(arr))
    }

    pub fn make_dict(key_sig: &str, val_sig: &str, map: DictMap) -> Result<Container> {
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
        map: DictMap,
    ) -> Result<Container> {
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
pub struct Variant {
    pub sig: signature::Type,
    pub value: Param,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Array {
    pub element_sig: signature::Type,
    pub values: Vec<Param>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dict {
    pub key_sig: signature::Base,
    pub value_sig: signature::Type,
    pub map: DictMap,
}

/// The Types a message can have as parameters
/// There are From<T> impls for most of the Base ones
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Param {
    Base(Base),
    Container(Container),
}
