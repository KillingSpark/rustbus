//! # Rustbus
//! Rustbus is a dbus library that allows for clients to perform method_calls on services on the bus or to implement your own service that listens on the bus.
//!
//! ## Quickstart
//! ```rust,no_run
//! use rustbus::{connection::Timeout, get_session_bus_path, DuplexConn, MessageBuilder};
//! fn main() -> Result<(), rustbus::connection::Error> {
//!     /// To get a connection going you need to connect to a bus. You will likely use either the session or the system bus.
//!     let session_path = get_session_bus_path()?;
//!     let mut con = DuplexConn::connect_to_bus(session_path, true)?;
//!     // Dont forget to send the obligatory hello message. send_hello wraps the call and parses the response for convenience.
//!     let unique_name = con.send_hello(Timeout::Infinite)?;
//!
//!     // Next you will probably want to create a new message to send out to the world
//!     let mut sig = MessageBuilder::new()
//!         .signal(
//!             "io.killing.spark".into(),
//!             "TestSignal".into(),
//!             "/io/killing/spark".into(),
//!         )
//!         .build();
//!     
//!     // To put parameters into that message you use the sig.body.push_param functions. These accept anything that can be marshalled into a dbus parameter
//!     // You can derive or manually implement that trait for your own types if you need that.
//!     sig.body.push_param("My cool new Signal!").unwrap();
//!     
//!     // Now send you signal to all that want to hear it!
//!     con.send.send_message(&mut sig, Timeout::Infinite)?;
//!     
//!     // To receive messages sent to you you can call the various functions on the RecvConn. The simplest is this:
//!     let message = con.recv.get_next_message(Timeout::Infinite)?;  
//!     
//!     // Now you can inspect the message.dynheader for all the metadata on the message
//!     println!("The messages dynamic header: {:?}", message.dynheader);
//!
//!     // After inspecting that dynheader you should know which content the message should contain
//!     let cool_string = message.body.parser().get::<&str>().unwrap();
//!     println!("Received a cool string: {}", cool_string);
//!     Ok(())
//! }
//! ```
//!
//! ## Other connection Types
//! There are some more connection types in the connection module. These are convenience wrappes around the concepts presented in the quickstart.
//! * RpcConn is meant for clients calling methods on services on the bus
//! * DispatchConn is meant for services that need to dispatch calls to many handlers.
//!
//! Since different usecases have different constraints you might need to write your own wrapper around the low level conn. This should not be too hard
//! if you copy the existing ones and modify them to your needs. If you have an issue that would be helpful for others I would of course consider adding
//! it to this libary.
//!
//! ## Params and Marshal and Unmarshal
//! This lib started out as an attempt to understand how dbus worked. Thus I modeled the types a closely as possible with enums, which is still in the params module.
//! This is kept around for weird weird edge-cases where that might be necessary but they should not generally be used.
//!
//! Instead you should be using the Marshal and Unmarshal traits which are implemented for most common types you will need. The idea is to map rust types
//! as closely as possible to dbus types. The trivial types like String and u64 etc are dealt with easily. For tuple-structs there are impls up to a
//! certain size. After that you'd need to copy the impl from this lib and extend it accordingly. This might be dealt with in the future if variadic generics get
//! added to rust.
//!
//! For structs there is a derive proc-macro that derives the necessary trait impls for you. Look into rustbus_derive if this is of need for you.
//!
//! For Variants there is a macro dbus_variant_sig! and dbus_variant_var! which will generate an enum and the Marshal and Unmarshal impls for you. These might get
//! replaced with a proc-macro derive like it exists already for structs.
//!
//! The doc for the traits gives more specifics on how to implement them for your own types if necessary.
//!
//! There is an exmaple for all of this in `examples/user_defined_types.rs`.
//! And for the deriving for structs there is an example in `examples/deriving.rs`
//!
//! ## Filedescriptors
//! Dbus can send filedescriptors around for you. Rustbus supports this. There is a special wrapper type in the wire module. This type tries to sensibly deal with
//! the pitfalls of sending and receiving filedescriptors in a sensible way. If you see any issues with the API or have wishes for extensions to the API please
//! open an issue.
//!
//! ## Byteorders
//! Dbus supports both big and little endian and so does rustbus. You can specify how a message should be marshalled when you create the MessageBuilder. Messages
//! can be received in any byteorder and will be transparently unmarshalled into the byteorder you CPU uses. Note that unmarshalling from/to the native byteorder will
//! be faster. The default byteorder is little endian.

pub mod auth;
pub mod connection;
pub mod message_builder;
pub mod params;
pub mod peer;
pub mod signature;
pub mod standard_messages;
pub mod wire;

// reexport derive macros
pub use rustbus_derive::*;

// TODO create a rustbus::prelude

// needed to make own filters in RpcConn
pub use message_builder::MessageType;

// needed to create a connection
pub use connection::dispatch_conn::DispatchConn;
pub use connection::ll_conn::DuplexConn;
pub use connection::ll_conn::RecvConn;
pub use connection::ll_conn::SendConn;
pub use connection::rpc_conn::RpcConn;
pub use connection::{get_session_bus_path, get_system_bus_path};

// needed to make new messages
pub use message_builder::{CallBuilder, MessageBuilder, SignalBuilder};
pub use wire::marshal::traits::Marshal;
pub use wire::marshal::traits::Signature;
pub use wire::unmarshal::traits::Unmarshal;

#[cfg(test)]
mod tests;

/// The supported byte orders
#[derive(Clone, Copy, Debug, PartialEq)]
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
    Marshal(crate::wire::marshal::Error),
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
