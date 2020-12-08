use rustbus::{connection::Timeout, get_session_bus_path, DuplexConn, MessageBuilder};

fn main() -> Result<(), rustbus::connection::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = DuplexConn::connect_to_bus(session_path, true)?;
    con.send_hello(Timeout::Infinite)?;

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
        .push_param4(
            100u8,
            vec!["ABCDE"].as_slice(),
            (162254319i32, "AABB", 20u8, false, "MyOwnedString"),
            (162254319i32, 30u8, 162254319i32),
        )
        .unwrap();

    sig.body
        .push_variant((162254319i32, "AABB", true, false, "MyOwnedString"))
        .unwrap();
    sig.body.push_param(100u8).unwrap();

    // Or we can add parameters later if we want to
    sig.body.push_param(&dict1).unwrap();

    println!("{:?}", sig);

    con.send.send_message(&mut sig, Timeout::Infinite)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    con.send.send_message(&mut sig, Timeout::Infinite)?;

    Ok(())
}
