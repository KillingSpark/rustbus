extern crate rustbus;
use rustbus::message;

fn main() {
    let mut con =
        rustbus::client_conn::Conn::connect_to_bus(std::path::PathBuf::from("/run/user/1000/bus"))
            .unwrap();

    let interface = "".to_owned();
    let member = "org.freedesktop.DBus.Hello".to_owned();
    let object = "/org/freedesktop/DBus".to_owned();
    let dest = "org.freedesktop.DBus".to_owned();

    let msg = message::Message::new(
        message::MessageType::Call,
        None,
        Some(member),
        Some(object),
        None,
        vec![message::Param::Base(message::Base::String(
            ":unique.name".to_owned(),
        ))],
    );

    println!("Send message: {:?}", msg);
    con.send_message(&msg).unwrap();
    let member = "org.freedesktop.DBus.AddMatch".to_owned();
    let object = "/org/freedesktop/DBus".to_owned();
    let dest = "org.freedesktop.DBus".to_owned();

    let msg = message::Message::new(
        message::MessageType::Call,
        None,
        Some(member),
        Some(object),
        Some(dest),
        vec![message::Param::Base(message::Base::String(
            "tyoe='signal'".to_owned(),
        ))],
    );
    println!("Send message: {:?}", msg);
    con.send_message(&msg).unwrap();
    
    println!("Wait for incoming messages");
    con.get_next_message().unwrap();
}
