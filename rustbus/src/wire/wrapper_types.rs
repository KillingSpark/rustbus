pub mod unixfd;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
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

#[derive(Debug, PartialEq)]
pub struct SignatureWrapper<'a>(&'a str);
impl<'a> SignatureWrapper<'a> {
    pub fn new(sig: &'a str) -> Result<Self, crate::params::validation::Error> {
        crate::params::validate_signature(sig)?;
        Ok(SignatureWrapper(sig))
    }
}
impl<'a> AsRef<str> for SignatureWrapper<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}
