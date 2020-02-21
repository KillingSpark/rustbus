//! # Rustbus
//! Rustbus is a dbus library that allows for RPC on services on the bus or to implement your own service that listens on the bus. There are some examples
//! in the src/bin directory but the gist is:
//!
//! ```rust,no_run
//! use rustbus::{get_session_bus_path, standard_messages, Conn, Container, DictMap, MessageBuilder};
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
//!     rpc_con.send_message(standard_messages::hello())?;
//!
//!     // Request a bus name if you want to
//!     rpc_con.send_message(standard_messages::request_name(
//!         "io.killing.spark".into(),
//!         0,
//!     ))?;
//!
//!     // send a signal to all bus members
//!     let sig = MessageBuilder::new()
//!     .signal(
//!         "io.killing.spark".into(),
//!         "TestSignal".into(),
//!         "/io/killing/spark".into(),
//!     )
//!     .with_params(vec![
//!         Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
//!     ])
//!     .build();
//!     rpc_con.send_message(sig)?;
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client_conn;
pub mod wire;
pub mod message;
pub mod message_builder;
pub mod signature;
pub mod standard_messages;

pub use client_conn::{get_session_bus_path, get_system_bus_path, Conn, RpcConn};
pub use message::{Container, DictMap, Message, MessageType};
pub use message_builder::{CallBuilder, MessageBuilder, SignalBuilder};

#[cfg(test)]
mod tests;
