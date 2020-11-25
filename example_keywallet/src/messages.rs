//! Data types needed for communication between service and client

use rustbus::wire::marshal::traits::ObjectPath;
use rustbus_derive::Marshal;
use rustbus_derive::Unmarshal;
use rustbus_derive::Signature;

#[derive(Marshal, Unmarshal, Signature, Clone)]
pub struct Secret<'a> {
    pub session: ObjectPath<'a>,
    pub params: Vec<u8>,
    pub value: Vec<u8>,
    pub content_type: String, 
}