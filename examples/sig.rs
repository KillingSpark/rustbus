use rustbus::{
    client_conn::Timeout, get_session_bus_path, standard_messages, Conn, MessageBuilder,
};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;

    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    let mut dict1 = std::collections::HashMap::new();
    dict1.insert("Key1", 100i32);
    dict1.insert("Key2", 200i32);

    // we can push up to 5 different types at once
    sig.body
        .push_param3(
            vec!["ABCDE"].as_slice(),
            (162254319i32, "AABB", true, false, "MyOwnedString"),
            (162254319i32, 162254319i32),
        )
        .unwrap();

    sig.body
        .push_variant((162254319i32, "AABB", true, false, "MyOwnedString"))
        .unwrap();

    // Or we can add parameters later if we want to
    sig.body.push_param(&dict1).unwrap();

    println!("{:?}", sig);

    con.send_message(&mut sig, Timeout::Infinite)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    con.send_message(&mut sig, Timeout::Infinite)?;

    Ok(())
}
