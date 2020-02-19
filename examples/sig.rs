use rustbus::{get_session_bus_path, standard_messages, Conn, Container, DictMap, MessageBuilder};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(standard_messages::hello())?;

    let mut dict_map = DictMap::new();
    dict_map.insert("Key1".to_owned().into(), 100i32.into());
    dict_map.insert("Key2".to_owned().into(), 200i32.into());

    let dict1 = Container::make_dict("s", "i", dict_map).unwrap();
    let dict2 = Container::make_dict("s", "(iiiiibbyy)", DictMap::new()).unwrap();

    let arr1 = Container::make_array("s", vec!["ABCDE".to_owned().into()]).unwrap();
    let arr2 = Container::make_array(
        "(is)",
        vec![
            Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
            Container::Struct(vec![305419896i32.into(), "CCDD".to_owned().into()]).into(),
        ],
    )
    .unwrap();
    let arr3 = Container::make_array("(sa{i(sisisis)}((si)uby))", vec![]).unwrap();

    let sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .with_params(vec![
            arr1.into(),
            Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
            arr2.into(),
            arr3.into(),
            dict1.into(),
            dict2.into(),
        ])
        .build();
    con.send_message(sig.clone())?;
    con.send_message(sig)?;

    Ok(())
}
