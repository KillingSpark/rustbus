extern crate rustbus;

fn main() {
    rustbus::client_conn::Conn::connect_to_bus(std::path::PathBuf::from("/run/user/1000/bus")).unwrap();
}