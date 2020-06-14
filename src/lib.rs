//! # Rustbus
//! Rustbus is a dbus library that allows for RPC on services on the bus or to implement your own service that listens on the bus. There are some examples
//! in the src/bin directory but the gist is:
//!
//! ```rust,no_run
//! use rustbus::{get_session_bus_path, standard_messages, Conn, Container, params::DictMap, MessageBuilder, client_conn::Timeout};
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
//! 2. Using the Marshal trait exported as rustbus::Marshal
//!
//! The first will work for any and everything you might want to marshal, but is a bit more work to
//! actually setup. It is also slower than the Marshal trait. So for most applications I would recommend the
//! newer, faster, and more ergonomic trait based approach.
//!
//! For receiving messages only rustbus::params approach is currently supported. I am currently working on improving
//! this. There are a few ways I could go about this, and I am exploring what works best.
//!

pub mod auth;
pub mod client_conn;
pub mod message;
pub mod message_builder;
pub mod params;
pub mod peer;
pub mod signature;
pub mod standard_messages;
pub mod wire;

// TODO create a rustbus::prelude

// needed to make own filters in RpcConn
pub use message::{Message, MessageType};

// needed to create a connection
pub use client_conn::{get_session_bus_path, get_system_bus_path, Conn, RpcConn};

// needed to make new messages
pub use message_builder::{CallBuilder, MessageBuilder, SignalBuilder};
pub use wire::marshal::traits::Marshal;
pub use wire::marshal::traits::Signature;

// needed for destructuring received messages
pub use params::{Base, Container, Param};

#[cfg(test)]
mod tests;

/// The supported byte orders
#[derive(Clone, Copy, Debug)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}
