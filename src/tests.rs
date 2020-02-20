use crate::marshal::marshal;
use crate::message::Base;
use crate::message::Param;
use crate::unmarshal::unmarshal_header;
use crate::unmarshal::unmarshal_next_message;

// this tests the happy path
#[test]
fn test_marshal_unmarshal() {
    let mut params: Vec<Param> = Vec::new();

    params.push(128u8.into());
    params.push(128u16.into());
    params.push((-128i16).into());
    params.push(1212128u32.into());
    params.push((-1212128i32).into());
    params.push(1212121212128u64.into());
    params.push((-1212121212128i64).into());
    params.push("TesttestTesttest".to_owned().into());
    params.push(Base::ObjectPath("/this/object/path".into()).into());

    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .with_params(params)
        .build();
    msg.serial = Some(1);
    let mut buf = Vec::new();
    marshal(&msg, crate::message::ByteOrder::LittleEndian, &[], &mut buf).unwrap();
    let (_, header) = unmarshal_header(&buf, 0).unwrap();

    let (_, unmarshed_msg) =
        unmarshal_next_message(&header, &buf, crate::unmarshal::HEADER_LEN).unwrap();

    assert_eq!(msg.params, unmarshed_msg.params);
}
