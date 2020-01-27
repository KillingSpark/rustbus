extern crate rustbus;
use rustbus::message;
use rustbus::message_builder::MessageBuilder;

fn main() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
    let mut rpc_con = rustbus::client_conn::RpcConn::new(con);

    let hello_msg = MessageBuilder::new()
        .call("Hello".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build();

    println!("Send message: {:?}", hello_msg);
    let hello_serial = rpc_con.send_message(hello_msg).unwrap().serial.unwrap();

    println!("\n");
    println!("\n");
    println!("\n");
    println!("Wait for incoming messages");
    let msg = rpc_con.wait_response(&hello_serial).unwrap();
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    //let member = "Ping".to_owned();
    //let interface = "org.freedesktop.DBus.Peer".to_owned();
    //let object = "/org/freedesktop/DBus".to_owned();
    //let mut ping_msg = message::Message::new(message::MessageType::Call, 1337);
    //ping_msg.set_object(object);
    //ping_msg.set_interface(interface);
    //ping_msg.set_member(member);
    //println!("Send message: {:?}", ping_msg);
    //con.send_message(&ping_msg).unwrap();

    //let member = "ListNames".to_owned();
    //let interface = "org.freedesktop.DBus".to_owned();
    //let object = "/org/freedesktop/DBus".to_owned();
    //let dest = "org.freedesktop.DBus".to_owned();
    //let mut list_msg = message::Message::new(message::MessageType::Call, 1338);
    //list_msg.set_object(object);
    //list_msg.set_interface(interface);
    //list_msg.set_member(member);
    //list_msg.set_destination(dest);
    //println!("Send message: {:?}", list_msg);
    //con.send_message(&list_msg).unwrap();

    let sig_listen_msg = MessageBuilder::new()
        .call("AddMatch".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .with_params(vec![message::Param::Base(message::Base::String(
            "type='signal'".to_owned(),
        ))])
        .at("org.freedesktop.DBus".into())
        .build();

    println!("Send message: {:?}", sig_listen_msg);
    rpc_con.send_message(sig_listen_msg).unwrap();

    loop {
        println!("Wait for incoming messages");
        let msg = rpc_con.wait_signal().unwrap();
        println!("Got signal: {:?}", msg);
        println!("\n");
        println!("\n");
        println!("\n");
    }
}
