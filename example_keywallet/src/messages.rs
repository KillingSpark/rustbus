//! Data types needed for communication between service and client

use rustbus::wire::ObjectPath;
use rustbus::Marshal;
use rustbus::Signature;
use rustbus::Unmarshal;

#[derive(Marshal, Unmarshal, Signature, Clone)]
pub struct Secret {
    pub session: ObjectPath<String>,
    pub params: Vec<u8>,
    pub value: Vec<u8>,
    pub content_type: String,
}
