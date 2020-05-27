use rustbus::{
    client_conn::Timeout, get_session_bus_path, signature, standard_messages, Conn, Container,
    MessageBuilder,
};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;

    // of course you can also build arrays with structs (and any deeper nesting you want)
    let arr2 = Container::make_array(
        "(is)",
        &mut vec![
            Container::Struct(vec![162254319i32.into(), "AABB".into()]),
            Container::Struct(vec![305419896i32.into(), "CCDD".into()]),
        ]
        .into_iter(),
    )
    .unwrap();

    // The shorthand using the string notation really comes in handy when the types get ridiculous
    let arr3 = Container::make_array_ref("(a{i(sisisis)}((si)uby))", &[]).unwrap();

    // But if you want you can create the signature yourself
    let arr4 =
        Container::make_array_ref_with_sig(signature::Type::Base(signature::Base::String), &[])
            .unwrap();

    // You can also avoid specifing the signature entirely. This requires at least one element to be present, else try_from will fail
    use std::convert::TryFrom;
    let element = Container::Struct(vec![162254319i32.into(), "Inferred type".into()]);
    let arr5 = Container::try_from(vec![element.into()]).unwrap();

    // Building a dictmap implicitly using the convert::From impls for the base types. This means giving up ownership ober the map
    // (You can clone it of course, to keep a copy around!)
    let mut dict1: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    dict1.insert("Key1", 100i32);
    dict1.insert("Key2", 200i32);

    // Now we can build a message from all of these
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    sig.body.push_param(vec!["ABCDE"].as_slice()).unwrap();
    sig.body
        .push_param((162254319i32, "AABB", true, false, "MyOwnedString"))
        .unwrap();
    //sig.body.push_param((162254319i32, 162254319i32)).unwrap();
    //sig.body
    //    .push_variant((162254319i32, "AABB", true, false, "MyOwnedString"))
    //    .unwrap();
//
    //// Or we can add parameters later if we want to
    //sig.body.push_param(&dict1).unwrap();

    println!("{:?}", sig);

    con.send_message(&mut sig, Timeout::Infinite)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    con.send_message(&mut sig, Timeout::Infinite)?;

    Ok(())
}
