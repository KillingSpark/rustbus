//! This module implemets the org.freedesktop.DBus.Peer API for the RpcConn
//!
//! This might be useful for users of this library, but is kept optional

mod peer_handling;
pub use peer_handling::*;
