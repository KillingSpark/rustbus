//! This serves as a testing ground for rustbus. It implements the secret-service API from freedesktop.org <https://specifications.freedesktop.org/secret-service/latest/>.
//! Note though that this is not meant as a real secret-service you should use, it will likely be very insecure. This is just to have a realworld
//! usecase to validate the existing codebase and new ideas

use rustbus::connection::get_session_bus_path;
use rustbus::connection::ll_conn::DuplexConn;
fn main() {
    let mut con = DuplexConn::connect_to_bus(get_session_bus_path().unwrap(), false).unwrap();

    con.send_hello(rustbus::connection::Timeout::Infinite)
        .unwrap();

    let resp = con
        .recv
        .get_next_message(rustbus::connection::Timeout::Infinite)
        .unwrap();

    println!("Unique name: {}", resp.body.parser().get::<&str>().unwrap());

    let mut rpc_conn = rustbus::connection::rpc_conn::RpcConn::new(con);
    let mut msg = rustbus::message_builder::MessageBuilder::new()
        .call("SearchItems")
        .on("/org/freedesktop/secrets")
        .with_interface("org.freedesktop.Secret.Service")
        .at("io.killingspark.secrets")
        .build();

    let attrs = std::collections::HashMap::<String, String>::new();
    msg.body.push_param(&attrs).unwrap();

    let serial = rpc_conn
        .send_message(&mut msg)
        .unwrap()
        .write_all()
        .unwrap();
    let resp = rpc_conn
        .wait_response(serial, rustbus::connection::Timeout::Infinite)
        .unwrap();
    println!("Header: {:?}", resp.dynheader);
    match resp.typ {
        rustbus::message_builder::MessageType::Error => {
            println!(
                "Error name: {}",
                resp.dynheader.error_name.as_ref().unwrap()
            );
            println!("Error: {}", resp.body.parser().get::<&str>().unwrap());
        }
        _ => {
            let (unlocked, locked) = resp
                .body
                .parser()
                .get2::<Vec<rustbus::wire::ObjectPath<&str>>, Vec<rustbus::wire::ObjectPath<&str>>>(
                )
                .unwrap();
            println!("Items found: (unlocked){:?} (locked){:?}", unlocked, locked);
        }
    };
}
