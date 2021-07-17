use std::convert::TryFrom;

pub mod unixfd;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
/// Wraps a String or a &str or whatever implements AsRef<str> and checks at creation, that it is a valid ObjectPath
pub struct ObjectPath<S: AsRef<str>>(S);
impl<S: AsRef<str>> ObjectPath<S> {
    pub fn new(path: S) -> Result<Self, crate::params::validation::Error> {
        crate::params::validate_object_path(path.as_ref())?;
        Ok(ObjectPath(path))
    }
    pub fn to_owned(&self) -> ObjectPath<String> {
        ObjectPath(self.as_ref().to_owned())
    }
}
impl<S: AsRef<str>> AsRef<str> for ObjectPath<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for ObjectPath<&'a str> {
    type Error = crate::params::validation::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        ObjectPath::<&'a str>::new(value)
    }
}

impl TryFrom<String> for ObjectPath<String> {
    type Error = crate::params::validation::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ObjectPath::<String>::new(value)
    }
}

#[derive(Debug, PartialEq)]
/// Wraps a String or a &str or whatever implements AsRef<str> and checks at creation, that it is a valid Signature
pub struct SignatureWrapper<S: AsRef<str>>(S);
impl<S: AsRef<str>> SignatureWrapper<S> {
    pub fn new(sig: S) -> Result<Self, crate::params::validation::Error> {
        crate::params::validate_signature(sig.as_ref())?;
        Ok(SignatureWrapper(sig))
    }
}
impl<S: AsRef<str>> AsRef<str> for SignatureWrapper<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for SignatureWrapper<&'a str> {
    type Error = crate::params::validation::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        SignatureWrapper::<&'a str>::new(value)
    }
}

impl TryFrom<String> for SignatureWrapper<String> {
    type Error = crate::params::validation::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SignatureWrapper::<String>::new(value)
    }
}
