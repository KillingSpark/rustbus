extern crate rustbus;
use rustbus::message::Base;
use rustbus::message::Container;
use rustbus::message::Param;
use rustbus::message::DictMap;
use rustbus::message_builder::MessageBuilder;

fn main() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let mut con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
    con.send_message(rustbus::standard_messages::hello()).unwrap();

    let sig = MessageBuilder::new()
        .signal("io.killing.spark".into(), "TestSignal".into(),  "/io/killing/spark".into())
        .with_params(vec![
            Container::Array(vec!["ABCDE".to_owned().into()]).into(),
            Container::Array(vec![162254319i32.into(), 305419896i32.into()]).into(),
            "ABCDE".to_owned().into()
        ])
        .build();
    con.send_message(sig.clone()).unwrap();
    con.send_message(sig).unwrap();
}
