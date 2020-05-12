use rustbus::{
    client_conn::Timeout, get_session_bus_path, params::DictMap, signature, standard_messages,
    Conn, Container, MessageBuilder,
};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;

    // To create a dict or array a signature is needed. You can use the string representation
    let dict3 = Container::make_dict("i", "i", (0..124i32).map(|v| (v, v + 1))).unwrap();
    let arr1 = Container::make_array("s", vec!["ABCDE"].into_iter()).unwrap();

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
    let mut dict_map: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    dict_map.insert("Key1", 100i32);
    dict_map.insert("Key2", 200i32);
    let _dict1 = Container::make_dict("s", "i", dict_map.clone().into_iter()).unwrap();
    let dict1 = Container::make_dict("s", "i", dict_map.into_iter()).unwrap();

    // Building a dictmap explicitly using the convert::From impls for the base types, and not giving up ownership
    let mut dict_map = DictMap::new();
    dict_map.insert("Key1".into(), 100i32.into());
    dict_map.insert("Key2".into(), 200i32.into());
    let dict2 = Container::make_dict_ref("s", "i", &dict_map).unwrap();

    // To create a more complex object, you have to write a bit more specific code
    let struct1 = Container::Struct(vec![
        162254319i32.into(),
        "AABB".into(),
        true.into(),
        false.into(),
        "MyOwnedString".to_owned().into(),
    ]);
    // But if you only have one type in there you can use a shorthand
    let struct2 = Container::make_struct(vec![162254319i32, 162254319i32]);
    // If you only have a few types there are shorthands for that too
    let mut struct3 = Container::make_struct3(162254319i32, 162254319u64, "Mixed Parameters");
    // If you have more parameters for that you can also push them one by one if you prefer that over struct1
    struct3.push(1234i64).unwrap();
    struct3.push(309845738u32).unwrap();
    struct3
        .push("Owned Strings are fine too btw".to_owned())
        .unwrap();

    // creating variants is very easy, just pass any Param into Container::make_variant
    let variant = Container::make_variant(Container::Struct(vec![
        162254319i32.into(),
        "Variant content".into(),
    ]));

    // Now we can build a message from all of these
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .with_params(vec![
            arr1.into(),
            struct1.into(),
            struct2.into(),
            struct3.into(),
            arr2.into(),
            arr3.into(),
            arr4.into(),
            arr5.into(),
            variant.into(),
        ])
        .build();

    // Or we can add parameters later if we want to
    sig.add_param3(dict1, dict2, dict3);

    con.send_message(&mut sig, Timeout::Infinite)?;
    con.send_message(&mut sig, Timeout::Infinite)?;

    Ok(())
}
