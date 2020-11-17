//! Data types needed for communication between service and client

use rustbus::wire::ObjectPath;
use rustbus_derive::Marshal;
use rustbus_derive::Unmarshal;

#[derive(Marshal, Unmarshal)]
struct Secret {
    session: ObjectPath,
    params: Vec<u8>,
    value: Vec<u8>,
    content_type: String, 
}