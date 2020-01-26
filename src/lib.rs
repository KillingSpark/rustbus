#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[macro_use]
extern crate nix;

pub mod client_conn;
pub mod marshal;
pub mod unmarshal;
pub mod message;
pub mod signature;
pub mod auth;