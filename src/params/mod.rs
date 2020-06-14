//! Map dbus concepts 1:1 to enums/structs
//!
//! This is for cases where the trait based (un-)marshalling does not work for you. It is a bit less effcient
//! and way less ergonomic but it allows to do everything dbus can do for you. It also allows for a more explorative approach
//! if you do not know what content to expect in received messages (e.g. you implement a tool similar to dbus-monitor).

mod container_constructors;
mod conversion;
pub mod message;
mod types;
pub mod validation;

pub use conversion::*;
pub use types::*;
pub use validation::*;
