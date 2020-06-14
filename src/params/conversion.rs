//! Provide a few Into<...> implementations to make working with Param a bit easier

use super::*;
use crate::signature;
use crate::Error;

impl<'a, 'e> Param<'a, 'e> {
    pub fn into_container(self) -> Result<Container<'a, 'e>, Param<'a, 'e>> {
        match self {
            Param::Base(_) => Err(self),
            Param::Container(c) => Ok(c),
        }
    }
    pub fn as_base(&'a self) -> Option<&'a Base<'a>> {
        match self {
            Param::Base(b) => Some(b),
            Param::Container(_) => None,
        }
    }
    pub fn as_base_mut(&'a mut self) -> Option<&'a mut Base<'a>> {
        match self {
            Param::Base(b) => Some(b),
            Param::Container(_) => None,
        }
    }
    pub fn as_slice(&'a self) -> Option<&'a [Param]> {
        match self {
            Param::Container(Container::Array(arr)) => Some(arr.values.as_slice()),
            Param::Container(Container::ArrayRef(arr)) => Some(&arr.values),
            Param::Container(Container::Struct(arr)) => Some(&arr),
            Param::Container(Container::StructRef(arr)) => Some(arr),
            _ => None,
        }
    }

    pub fn as_str(&'a self) -> Option<&'a str> {
        match self {
            Param::Base(Base::String(b)) => Some(b),
            Param::Base(Base::StringRef(b)) => Some(b),
            _ => None,
        }
    }

    // unsiged ints

    pub fn as_u64(&'a self) -> Option<&'a u64> {
        match self {
            Param::Base(Base::Uint64(b)) => Some(b),
            Param::Base(Base::Uint64Ref(b)) => Some(b),
            _ => None,
        }
    }
    pub fn as_u32(&'a self) -> Option<&'a u32> {
        match self {
            Param::Base(Base::Uint32(b)) => Some(b),
            _ => None,
        }
    }
    pub fn as_u16(&'a self) -> Option<&'a u16> {
        match self {
            Param::Base(Base::Uint16(b)) => Some(b),
            _ => None,
        }
    }

    // signed ints

    pub fn as_i64(&'a self) -> Option<&'a i64> {
        match self {
            Param::Base(Base::Int64(b)) => Some(b),
            _ => None,
        }
    }
    pub fn as_i32(&'a self) -> Option<&'a i32> {
        match self {
            Param::Base(Base::Int32(b)) => Some(b),
            _ => None,
        }
    }
    pub fn as_i16(&'a self) -> Option<&'a i16> {
        match self {
            Param::Base(Base::Int16(b)) => Some(b),
            _ => None,
        }
    }

    // special stuff

    pub fn as_byte(&'a self) -> Option<&'a u8> {
        match self {
            Param::Base(Base::Byte(b)) => Some(b),
            Param::Base(Base::ByteRef(b)) => Some(b),
            _ => None,
        }
    }
    pub fn as_bool(&'a self) -> Option<&'a bool> {
        match self {
            Param::Base(Base::Boolean(b)) => Some(b),
            Param::Base(Base::BooleanRef(b)) => Some(b),
            _ => None,
        }
    }
    pub fn as_unix_fd_idx(&'a self) -> Option<&'a u32> {
        match self {
            Param::Base(Base::UnixFd(b)) => Some(b),
            Param::Base(Base::UnixFdRef(b)) => Some(b),
            _ => None,
        }
    }

    pub fn into_string(self) -> Result<String, Param<'a, 'e>> {
        match self {
            Param::Base(Base::String(s)) => Ok(s),
            _ => Err(self),
        }
    }

    pub fn into_str(self) -> Result<&'a str, Param<'a, 'e>> {
        match self {
            Param::Base(Base::StringRef(s)) => Ok(s),
            _ => Err(self),
        }
    }

    pub fn into_u64(self) -> Result<u64, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Uint64(s)) => Ok(s),
            Param::Base(Base::Uint64Ref(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_u32(self) -> Result<u32, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Uint32(s)) => Ok(s),
            Param::Base(Base::Uint32Ref(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_u16(self) -> Result<u16, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Uint16(s)) => Ok(s),
            Param::Base(Base::Uint16Ref(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_i64(self) -> Result<i64, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Int64(s)) => Ok(s),
            Param::Base(Base::Int64Ref(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_i32(self) -> Result<i32, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Int32(s)) => Ok(s),
            Param::Base(Base::Int32Ref(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_i16(self) -> Result<i16, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Int16(s)) => Ok(s),
            Param::Base(Base::Int16Ref(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_byte(self) -> Result<u8, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Byte(s)) => Ok(s),
            Param::Base(Base::ByteRef(s)) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_bool(self) -> Result<bool, Param<'a, 'e>> {
        match self {
            Param::Base(Base::Boolean(s)) => Ok(s),
            Param::Base(Base::BooleanRef(s)) => Ok(*s),
            _ => Err(self),
        }
    }
}

//
//
// Base TO
//
//

impl<'a> Base<'a> {
    pub fn as_str(&'a self) -> Option<&'a str> {
        match self {
            Base::String(b) => Some(b),
            Base::StringRef(b) => Some(b),
            _ => None,
        }
    }

    // unsiged ints

    pub fn as_u64(&'a self) -> Option<&'a u64> {
        match self {
            Base::Uint64(b) => Some(b),
            Base::Uint64Ref(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_u32(&'a self) -> Option<&'a u32> {
        match self {
            Base::Uint32(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_u16(&'a self) -> Option<&'a u16> {
        match self {
            Base::Uint16(b) => Some(b),
            _ => None,
        }
    }

    // signed ints

    pub fn as_i64(&'a self) -> Option<&'a i64> {
        match self {
            Base::Int64(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_i32(&'a self) -> Option<&'a i32> {
        match self {
            Base::Int32(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_i16(&'a self) -> Option<&'a i16> {
        match self {
            Base::Int16(b) => Some(b),
            _ => None,
        }
    }

    // special stuff

    pub fn as_byte(&'a self) -> Option<&'a u8> {
        match self {
            Base::Byte(b) => Some(b),
            Base::ByteRef(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_bool(&'a self) -> Option<&'a bool> {
        match self {
            Base::Boolean(b) => Some(b),
            Base::BooleanRef(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_unix_fd_idx(&'a self) -> Option<&'a u32> {
        match self {
            Base::UnixFd(b) => Some(b),
            Base::UnixFdRef(b) => Some(b),
            _ => None,
        }
    }

    pub fn into_string(self) -> Result<String, Self> {
        match self {
            Base::String(s) => Ok(s),
            _ => Err(self),
        }
    }

    pub fn into_str(self) -> Result<&'a str, Self> {
        match self {
            Base::StringRef(s) => Ok(s),
            _ => Err(self),
        }
    }

    pub fn into_u64(self) -> Result<u64, Self> {
        match self {
            Base::Uint64(s) => Ok(s),
            Base::Uint64Ref(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_u32(self) -> Result<u32, Self> {
        match self {
            Base::Uint32(s) => Ok(s),
            Base::Uint32Ref(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_u16(self) -> Result<u16, Self> {
        match self {
            Base::Uint16(s) => Ok(s),
            Base::Uint16Ref(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_i64(self) -> Result<i64, Self> {
        match self {
            Base::Int64(s) => Ok(s),
            Base::Int64Ref(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_i32(self) -> Result<i32, Self> {
        match self {
            Base::Int32(s) => Ok(s),
            Base::Int32Ref(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_i16(self) -> Result<i16, Self> {
        match self {
            Base::Int16(s) => Ok(s),
            Base::Int16Ref(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_byte(self) -> Result<u8, Self> {
        match self {
            Base::Byte(s) => Ok(s),
            Base::ByteRef(s) => Ok(*s),
            _ => Err(self),
        }
    }
    pub fn into_bool(self) -> Result<bool, Self> {
        match self {
            Base::Boolean(s) => Ok(s),
            Base::BooleanRef(s) => Ok(*s),
            _ => Err(self),
        }
    }
}

impl<'a> std::convert::From<&Base<'a>> for signature::Base {
    fn from(b: &Base) -> crate::signature::Base {
        match b {
            Base::Boolean(_) => signature::Base::Boolean,
            Base::Byte(_) => signature::Base::Byte,
            Base::Double(_) => signature::Base::Double,
            Base::Int16(_) => signature::Base::Int16,
            Base::Int32(_) => signature::Base::Int32,
            Base::Int64(_) => signature::Base::Int64,
            Base::Uint16(_) => signature::Base::Uint16,
            Base::Uint32(_) => signature::Base::Uint32,
            Base::Uint64(_) => signature::Base::Uint64,
            Base::ObjectPath(_) => signature::Base::ObjectPath,
            Base::Signature(_) => signature::Base::Signature,
            Base::String(_) => signature::Base::String,
            Base::UnixFd(_) => signature::Base::UnixFd,
            Base::BooleanRef(_) => signature::Base::Boolean,
            Base::ByteRef(_) => signature::Base::Byte,
            Base::DoubleRef(_) => signature::Base::Double,
            Base::Int16Ref(_) => signature::Base::Int16,
            Base::Int32Ref(_) => signature::Base::Int32,
            Base::Int64Ref(_) => signature::Base::Int64,
            Base::Uint16Ref(_) => signature::Base::Uint16,
            Base::Uint32Ref(_) => signature::Base::Uint32,
            Base::Uint64Ref(_) => signature::Base::Uint64,
            Base::ObjectPathRef(_) => signature::Base::ObjectPath,
            Base::SignatureRef(_) => signature::Base::Signature,
            Base::StringRef(_) => signature::Base::String,
            Base::UnixFdRef(_) => signature::Base::UnixFd,
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for bool {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<bool, Error> {
        if let Base::Boolean(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for String {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<String, Error> {
        if let Base::String(value) = b {
            Ok(value.clone())
        } else {
            Err(Error::InvalidType)
        }
    }
}
impl<'a> std::convert::TryFrom<&Base<'a>> for &'a str {
    type Error = Error;
    fn try_from(b: &Base<'a>) -> std::result::Result<&'a str, Error> {
        if let Base::StringRef(value) = b {
            Ok(value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for u8 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u8, Error> {
        if let Base::Byte(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for u16 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u16, Error> {
        if let Base::Uint16(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for u32 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u32, Error> {
        if let Base::Uint32(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for u64 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u64, Error> {
        if let Base::Uint64(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for i16 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i16, Error> {
        if let Base::Int16(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for i32 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i32, Error> {
        if let Base::Int32(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl<'a> std::convert::TryFrom<&Base<'a>> for i64 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i64, Error> {
        if let Base::Int64(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

//
//
// Param TO
//
//

impl<'a, 'e> std::convert::From<&Param<'a, 'e>> for signature::Type {
    fn from(p: &Param<'a, 'e>) -> crate::signature::Type {
        match p {
            Param::Base(b) => signature::Type::Base(b.into()),
            Param::Container(c) => signature::Type::Container(c.into()),
        }
    }
}

//
//
// Param FROM
//
//

impl<'a, 'e, B: Into<Base<'a>>> std::convert::From<B> for Param<'a, 'e> {
    fn from(s: B) -> Self {
        Param::Base(s.into())
    }
}

impl<'a, 'e> std::convert::From<Container<'a, 'e>> for Param<'a, 'e> {
    fn from(s: Container<'a, 'e>) -> Self {
        Param::Container(s)
    }
}

//
//
// Container FROM
//
//

impl<'a, 'e> std::convert::TryFrom<(signature::Type, Vec<Param<'a, 'e>>)> for Container<'a, 'e> {
    type Error = Error;
    fn try_from(
        parts: (signature::Type, Vec<Param<'a, 'e>>),
    ) -> std::result::Result<Container<'a, 'e>, Error> {
        let arr = Array {
            element_sig: parts.0,
            values: parts.1,
        };
        validate_array(&arr.values, &arr.element_sig)?;
        Ok(Container::Array(arr))
    }
}
impl<'a, 'e> std::convert::TryFrom<Vec<Param<'a, 'e>>> for Container<'a, 'e> {
    type Error = Error;
    fn try_from(elems: Vec<Param<'a, 'e>>) -> std::result::Result<Container<'a, 'e>, Error> {
        if elems.is_empty() {
            return Err(Error::EmptyArray);
        }
        Container::try_from((elems[0].sig(), elems))
    }
}

impl<'a, 'e> std::convert::TryFrom<(signature::Base, signature::Type, DictMap<'a, 'e>)>
    for Container<'a, 'e>
{
    type Error = Error;
    fn try_from(
        parts: (signature::Base, signature::Type, DictMap<'a, 'e>),
    ) -> std::result::Result<Container<'a, 'e>, Error> {
        let dict = Dict {
            key_sig: parts.0,
            value_sig: parts.1,
            map: parts.2,
        };
        validate_dict(&dict.map, dict.key_sig, &dict.value_sig)?;
        Ok(Container::Dict(dict))
    }
}
impl<'a, 'e> std::convert::TryFrom<DictMap<'a, 'e>> for Container<'a, 'e> {
    type Error = Error;
    fn try_from(elems: DictMap<'a, 'e>) -> std::result::Result<Container<'a, 'e>, Error> {
        if elems.is_empty() {
            return Err(Error::EmptyDict);
        }
        let key_sig = elems.keys().next().unwrap().sig();
        let value_sig = elems.values().next().unwrap().sig();

        if let signature::Type::Base(key_sig) = key_sig {
            Container::try_from((key_sig, value_sig, elems))
        } else {
            Err(crate::signature::Error::ShouldBeBaseType.into())
        }
    }
}

//
//
// Base FROM
//
//

impl<'a> std::convert::From<String> for Base<'a> {
    fn from(s: String) -> Self {
        Base::String(s)
    }
}
impl<'a> std::convert::From<&'a str> for Base<'a> {
    fn from(s: &'a str) -> Self {
        Base::StringRef(s)
    }
}
impl<'a> std::convert::From<bool> for Base<'a> {
    fn from(s: bool) -> Self {
        Base::Boolean(s)
    }
}
impl<'a> std::convert::From<u8> for Base<'a> {
    fn from(s: u8) -> Self {
        Base::Byte(s)
    }
}
impl<'a> std::convert::From<u16> for Base<'a> {
    fn from(s: u16) -> Self {
        Base::Uint16(s)
    }
}
impl<'a> std::convert::From<u32> for Base<'a> {
    fn from(s: u32) -> Self {
        Base::Uint32(s)
    }
}
impl<'a> std::convert::From<u64> for Base<'a> {
    fn from(s: u64) -> Self {
        Base::Uint64(s)
    }
}
impl<'a> std::convert::From<i16> for Base<'a> {
    fn from(s: i16) -> Self {
        Base::Int16(s)
    }
}
impl<'a> std::convert::From<i32> for Base<'a> {
    fn from(s: i32) -> Self {
        Base::Int32(s)
    }
}
impl<'a> std::convert::From<i64> for Base<'a> {
    fn from(s: i64) -> Self {
        Base::Int64(s)
    }
}
impl<'a> std::convert::From<&'a bool> for Base<'a> {
    fn from(s: &'a bool) -> Self {
        Base::BooleanRef(s)
    }
}
impl<'a> std::convert::From<&'a u8> for Base<'a> {
    fn from(s: &'a u8) -> Self {
        Base::ByteRef(s)
    }
}
impl<'a> std::convert::From<&'a u16> for Base<'a> {
    fn from(s: &'a u16) -> Self {
        Base::Uint16Ref(s)
    }
}
impl<'a> std::convert::From<&'a u32> for Base<'a> {
    fn from(s: &'a u32) -> Self {
        Base::Uint32Ref(s)
    }
}
impl<'a> std::convert::From<&'a u64> for Base<'a> {
    fn from(s: &'a u64) -> Self {
        Base::Uint64Ref(s)
    }
}
impl<'a> std::convert::From<&'a i16> for Base<'a> {
    fn from(s: &'a i16) -> Self {
        Base::Int16Ref(s)
    }
}
impl<'a> std::convert::From<&'a i32> for Base<'a> {
    fn from(s: &'a i32) -> Self {
        Base::Int32Ref(s)
    }
}
impl<'a> std::convert::From<&'a i64> for Base<'a> {
    fn from(s: &'a i64) -> Self {
        Base::Int64Ref(s)
    }
}

//
//
// Container TO
//
//

impl<'a, 'e> std::convert::From<&Container<'a, 'e>> for signature::Container {
    fn from(c: &Container<'a, 'e>) -> crate::signature::Container {
        match c {
            Container::Array(arr) => signature::Container::Array(Box::new(arr.element_sig.clone())),
            Container::Dict(dict) => {
                signature::Container::Dict(dict.key_sig, Box::new(dict.value_sig.clone()))
            }
            Container::Struct(params) => {
                signature::Container::Struct(params.iter().map(|param| param.into()).collect())
            }
            Container::Variant(_) => signature::Container::Variant,
            Container::ArrayRef(arr) => {
                signature::Container::Array(Box::new(arr.element_sig.clone()))
            }
            Container::DictRef(dict) => {
                signature::Container::Dict(dict.key_sig, Box::new(dict.value_sig.clone()))
            }
            Container::StructRef(params) => {
                signature::Container::Struct(params.iter().map(|param| param.into()).collect())
            }
        }
    }
}
