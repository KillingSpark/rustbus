extern crate rustbus;

fn main() {
    let mut con = rustbus::client_conn::Conn::connect_to_bus(std::path::PathBuf::from("/run/user/1000/bus")).unwrap();
    println!("Wait for incoming messages");
    con.get_next_message().unwrap();
}