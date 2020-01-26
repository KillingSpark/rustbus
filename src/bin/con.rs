extern crate rustbus;
use rustbus::message;
use rustbus::message_builder::MessageBuilder;

fn main() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let mut con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();

    let hello_msg = MessageBuilder::new()
        .call("Hello".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build();

    println!("Send message: {:?}", hello_msg);
    con.send_message(hello_msg).unwrap();

    println!("\n");
    println!("\n");
    println!("\n");
    println!("Wait for incoming messages");
    let msg = con.get_next_message().unwrap();
    println!("Got message: {:?}", msg);
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
    con.send_message(sig_listen_msg).unwrap();

    loop {
        println!("Wait for incoming messages");
        let msg = con.get_next_message().unwrap();
        println!("Got message: {:?}", msg);
        println!("\n");
        println!("\n");
        println!("\n");
    }
}
