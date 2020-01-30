//! # Rustbus
//! Rustbus is a dbus library that allows for RPC on services on the bus or to implement your own service that listens on the bus. There are some examples
//! in the src/bin directory but the gist is:
//!
//! ```
//! // Connect to the session bus
//! let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
//! let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
//! 
//! // Wrap the con in an RpcConnection which provides many convenient functions
//! let mut rpc_con = rustbus::client_conn::RpcConn::new(con);
//! 
//! // send the obligatory hello message
//! rpc_con.send_message(standard_messages::hello()).unwrap();
//! 
//! // Request a bus name if you want to
//! rpc_con.send_message(standard_messages::request_name(
//!     "io.killing.spark".into(),
//!     0,
//! ))
//! .unwrap();
//! 
//! // send a signal to all bus members
//! let sig = MessageBuilder::new()
//! .signal(
//!     "io.killing.spark".into(),
//!     "TestSignal".into(),
//!     "/io/killing/spark".into(),
//! )
//! .with_params(vec![
//!     Container::Array(vec!["ABCDE".to_owned().into()]).into(),
//!     Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
//!     Container::Array(vec![
//!         Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
//!         Container::Struct(vec![305419896i32.into(), "CCDD".to_owned().into()]).into(),
//!     ])
//!     .into(),
//!     Container::Dict(dict).into(),
//! ])
//! .build();
//! con.send_message(sig).unwrap();
//! ```

#[macro_use]
extern crate nix;

pub mod auth;
pub mod client_conn;
pub mod marshal;
pub mod message;
pub mod message_builder;
pub mod signature;
pub mod standard_messages;
pub mod unmarshal;
