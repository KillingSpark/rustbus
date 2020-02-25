use rustbus::{
    get_session_bus_path, params::DictMap, params::Param, params::Variant, signature,
    standard_messages, Conn, Container, MessageBuilder,
};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(standard_messages::hello(), None)?;

    // Building a dictmap using the convert::From impls for the base types
    let mut dict_map = DictMap::new();
    dict_map.insert("Key1".to_owned().into(), 100i32.into());
    dict_map.insert("Key2".to_owned().into(), 200i32.into());
    let dict1 = Container::make_dict("s", "i", dict_map).unwrap();

    // To create a more complex object, you have to write a bit more specific code
    let struct1 = Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]);

    // To create a dict or array a type is needed. You can use the string representation
    let dict2 = Container::make_dict("s", "(iiiiibbyy)", DictMap::new()).unwrap();

    let arr1 = Container::make_array("s", vec!["ABCDE".to_owned().into()]).unwrap();

    // of course you can also build arrays with structs (and any deeper nesting you want)
    let arr2 = Container::make_array(
        "(is)",
        vec![
            Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
            Container::Struct(vec![305419896i32.into(), "CCDD".to_owned().into()]).into(),
        ],
    )
    .unwrap();

    // The shorthand using the string notation really comes in handy when the types get ridiculous
    let arr3 = Container::make_array("(a{i(sisisis)}((si)uby))", vec![]).unwrap();

    // But if you want you can create the signature yourself
    let arr4 =
        Container::make_array_with_sig(signature::Type::Base(signature::Base::String), vec![])
            .unwrap();

    // You can also avoid specifing the signature entirely. This requires at least one element to be present, else try_from will fail
    use std::convert::TryFrom;
    let element: Param =
        Container::Struct(vec![162254319i32.into(), "Inferred type".to_owned().into()]).into();
    let arr5 = Container::try_from(vec![element]).unwrap();

    let variant_param: Param = Container::Struct(vec![
        162254319i32.into(),
        "Variant content".to_owned().into(),
    ])
    .into();
    let variant: Param = Container::Variant(Box::new(Variant {
        sig: variant_param.sig(),
        value: variant_param,
    }))
    .into();

    // Now we can build a message from all of these
    let sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .with_params(vec![
            arr1.into(),
            struct1.into(),
            arr2.into(),
            arr3.into(),
            arr4.into(),
            arr5.into(),
            dict1.into(),
            dict2.into(),
            variant,
        ])
        .build();
    con.send_message(sig.clone(), None)?;
    con.send_message(sig, None)?;

    Ok(())
}
