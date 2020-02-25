use super::*;
use crate::message::Error;
use crate::signature;

//
//
// Base TO
//
//

impl std::convert::From<&Base> for signature::Base {
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
        }
    }
}

impl std::convert::TryFrom<&Base> for bool {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<bool, Error> {
        if let Base::Boolean(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for String {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<String, Error> {
        if let Base::String(value) = b {
            Ok(value.clone())
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u8 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u8, Error> {
        if let Base::Byte(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u16 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u16, Error> {
        if let Base::Uint16(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u32 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u32, Error> {
        if let Base::Uint32(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u64 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u64, Error> {
        if let Base::Uint64(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for i16 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i16, Error> {
        if let Base::Int16(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for i32 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i32, Error> {
        if let Base::Int32(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for i64 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i64, Error> {
        if let Base::Int64(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}
