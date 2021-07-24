//! This serves as a testing ground for rustbus. It implements the secret-service API from freedesktop.org <https://specifications.freedesktop.org/secret-service/latest/>.
//! Note though that this is not meant as a real secret-service you should use, it will likely be very insecure. This is just to have a realworld
//! usecase to validate the existing codebase and new ideas

#[derive(Clone)]
pub struct Secret {
    pub params: Vec<u8>,
    pub value: Vec<u8>,
    pub content_type: String,
}

#[derive(Eq, PartialEq, Clone)]
pub struct LookupAttribute {
    pub name: String,
    pub value: String,
}

#[derive(Copy, Clone)]
pub enum LockState {
    Locked,
    Unlocked,
}

pub mod messages;
