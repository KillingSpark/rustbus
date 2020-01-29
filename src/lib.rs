#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

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
