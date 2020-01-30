extern crate rustbus;
use rustbus::message::Base;
use rustbus::message::Container;
use rustbus::message::DictMap;
use rustbus::message::Param;
use rustbus::message_builder::MessageBuilder;

fn main() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let mut con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
    con.send_message(rustbus::standard_messages::hello())
        .unwrap();

    let mut dict = DictMap::new();
    dict.insert("Key1".to_owned().into(), 100i32.into());
    dict.insert("Key2".to_owned().into(), 100i32.into());

    let sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .with_params(vec![
            Container::Array(vec!["ABCDE".to_owned().into()]).into(),
            Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
            Container::Array(vec![
                Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
                Container::Struct(vec![305419896i32.into(), "CCDD".to_owned().into()]).into(),
            ]).into(),
            Container::Dict(dict).into(),
        ])
        .build();
    con.send_message(sig.clone()).unwrap();
    con.send_message(sig).unwrap();
}
