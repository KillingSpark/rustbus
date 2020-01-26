extern crate rustbus;
use rustbus::message;

fn main() {
    let mut con =
        rustbus::client_conn::Conn::connect_to_bus(std::path::PathBuf::from("/run/user/1000/bus"))
            .unwrap();

    let interface = "org.freedesktop.DBus".to_owned();
    let member = "Hello".to_owned();
    let object = "/org/freedesktop/DBus".to_owned();
    let dest = "org.freedesktop.DBus".to_owned();

    let mut hello_msg = message::Message::new(message::MessageType::Call, 1);
    hello_msg.set_destination(dest);
    hello_msg.set_object(object);
    hello_msg.set_interface(interface);
    hello_msg.set_member(member);

    println!("Send message: {:?}", hello_msg);
    con.send_message(&hello_msg).unwrap();

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

    let member = "AddMatch".to_owned();
    let interface = "org.freedesktop.DBus".to_owned();
    let object = "/org/freedesktop/DBus".to_owned();
    let dest = "org.freedesktop.DBus".to_owned();
    let mut sig_listen_msg = message::Message::new(message::MessageType::Call, 1339);
    sig_listen_msg.set_object(object);
    sig_listen_msg.set_interface(interface);
    sig_listen_msg.set_member(member);
    sig_listen_msg.set_destination(dest);
    sig_listen_msg.push_params(vec![message::Param::Base(message::Base::String(
        "type='signal'".to_owned(),
    ))]);
    println!("Send message: {:?}", sig_listen_msg);
    con.send_message(&sig_listen_msg).unwrap();


    loop {
        println!("Wait for incoming messages");
        let msg = con.get_next_message().unwrap();
        println!("Got message: {:?}", msg);
        println!("\n");
        println!("\n");
        println!("\n");
    }
}
