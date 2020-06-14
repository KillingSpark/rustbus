//! # Rustbus
//! Rustbus is a dbus library that allows for RPC on services on the bus or to implement your own service that listens on the bus. There are some examples
//! in the src/bin directory but the gist is:
//!
//! ```rust,no_run
//! use rustbus::{get_session_bus_path, standard_messages, Conn, MessageBuilder, client_conn::Timeout};
//!
//! fn main() -> Result<(), rustbus::client_conn::Error> {
//!     // Connect to the session bus
//!     let session_path = get_session_bus_path()?;
//!     let con = Conn::connect_to_bus(session_path, true)?;
//!
//!     // Wrap the con in an RpcConnection which provides many convenient functions
//!     let mut rpc_con = rustbus::client_conn::RpcConn::new(con);
//!
//!     // send the obligatory hello message
//!     rpc_con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;
//!
//!     // Request a bus name if you want to
//!     rpc_con.send_message(&mut standard_messages::request_name(
//!         "io.killing.spark".into(),
//!         0,
//!     ), Timeout::Infinite)?;
//!
//!     // send a signal to all bus members
//!     let mut sig = MessageBuilder::new()
//!     .signal(
//!         "io.killing.spark".into(),
//!         "TestSignal".into(),
//!         "/io/killing/spark".into(),
//!     )
//!     .build();
//!     
//!     sig.body.push_param("Signal message!").unwrap();
//!     rpc_con.send_message(&mut sig, Timeout::Infinite)?;
//!     Ok(())
//! }
//! ```
//!
//! To add parameters to messages there are currently two possibilities:
//! 1. Using the explicit nested structs/enums from rustbus::params
//! 2. Using the (Un-)Marshal trait exported as rustbus::(Un-)Marshal
//!
//! The first will work for any and everything you might want to marshal, but is a bit more work to
//! actually setup. It is also slower than the Marshal trait. So for most applications I would recommend the
//! newer, faster, and more ergonomic trait based approach.
pub mod auth;
pub mod client_conn;
pub mod message_builder;
pub mod params;
pub mod peer;
pub mod signature;
pub mod standard_messages;
pub mod wire;

// TODO create a rustbus::prelude

// needed to make own filters in RpcConn
pub use message_builder::MessageType;

// needed to create a connection
pub use client_conn::{get_session_bus_path, get_system_bus_path, Conn, RpcConn};

// needed to make new messages
pub use message_builder::{CallBuilder, MessageBuilder, SignalBuilder};
pub use wire::marshal::traits::Marshal;
pub use wire::marshal::traits::Signature;
pub use wire::unmarshal::traits::Unmarshal;

#[cfg(test)]
mod tests;

/// The supported byte orders
#[derive(Clone, Copy, Debug)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

/// The different errors that can occur when dealing with messages
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidType,
    EmptyArray,
    EmptyDict,
    StringContainsNullByte,
    Unmarshal(crate::wire::unmarshal::Error),
    Validation(crate::params::validation::Error),
}

impl From<crate::params::validation::Error> for Error {
    fn from(e: crate::params::validation::Error) -> Self {
        Error::Validation(e)
    }
}
impl From<crate::wire::unmarshal::Error> for Error {
    fn from(e: crate::wire::unmarshal::Error) -> Self {
        Error::Unmarshal(e)
    }
}
impl From<crate::signature::Error> for Error {
    fn from(e: crate::signature::Error) -> Self {
        Error::Validation(crate::params::validation::Error::InvalidSignature(e))
    }
}
