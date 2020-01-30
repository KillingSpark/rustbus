use rustbus::{get_session_bus_path, standard_messages, Conn, Container, DictMap, MessageBuilder};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(standard_messages::hello())?;

    let mut dict = DictMap::new();
    dict.insert("Key1".to_owned().into(), 100i32.into());
    dict.insert("Key2".to_owned().into(), 200i32.into());

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
            ])
            .into(),
            Container::Dict(dict).into(),
        ])
        .build();
    con.send_message(sig.clone())?;
    con.send_message(sig)?;

    Ok(())
}
