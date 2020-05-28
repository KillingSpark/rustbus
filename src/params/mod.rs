//! The params is mostly for receiving messages. To make your own messages to send out you probably want to use the 
//! Marshal trait. If the trait does not work for you for some reason, you can build your message with the params.
//! There is a Marshal implementation for all Params.

mod container_constructors;
mod conversion;
mod types;
mod validation;

pub use conversion::*;
pub use types::*;
pub use validation::*;
