use crate::params::Base;
use crate::params::Param;
use crate::wire::marshal::marshal;
use crate::wire::unmarshal::unmarshal_dynamic_header;
use crate::wire::unmarshal::unmarshal_header;
use crate::wire::unmarshal::unmarshal_next_message;

mod dbus_send;
mod fdpassing;
mod verify_marshalling;
mod verify_padding;

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
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();

    // mixing old and new style of params and check that they are unmarshalled correctly
    msg.body.push_old_params(&params).unwrap();
    msg.body.push_param(128u8).unwrap();
    msg.body.push_param(128u64).unwrap();
    msg.body.push_param(128i32).unwrap();

    params.push(128u8.into());
    params.push(128u64.into());
    params.push(128i32.into());

    msg.dynheader.serial = Some(1);
    let mut buf = Vec::new();
    marshal(&msg, 0, &mut buf).unwrap();
    let (hdrbytes, header) = unmarshal_header(&buf, 0).unwrap();
    let (dynhdrbytes, dynheader) = unmarshal_dynamic_header(&header, &buf, hdrbytes).unwrap();

    let headers_plus_padding = hdrbytes + dynhdrbytes + (8 - ((hdrbytes + dynhdrbytes) % 8));
    assert_eq!(headers_plus_padding, buf.len());

    let (_, unmarshed_msg) = unmarshal_next_message(&header, dynheader, msg.get_buf(), 0).unwrap();

    let msg = unmarshed_msg.unmarshall_all().unwrap();

    assert_eq!(params, msg.params);
}

// this tests that invalid inputs return appropriate errors
#[test]
fn test_invalid_stuff() {
    // invalid signature
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();

    let err = msg
        .body
        .push_old_param(&Param::Base(Base::Signature("((((((((}}}}}}}".into())));
    assert_eq!(
        Err(crate::Error::Validation(
            crate::params::validation::Error::InvalidSignature(
                crate::signature::Error::InvalidSignature
            )
        )),
        err
    );

    // invalid objectpath
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();
    let err = msg
        .body
        .push_old_param(&Param::Base(Base::ObjectPath("invalid/object/path".into())));
    assert_eq!(
        Err(crate::Error::Validation(
            crate::params::validation::Error::InvalidObjectPath
        )),
        err
    );

    // invalid interface
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(".......io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();
    msg.dynheader.serial = Some(1);
    let mut buf = Vec::new();
    assert_eq!(
        Err(crate::Error::Validation(
            crate::params::validation::Error::InvalidInterface
        )),
        marshal(&msg, 0, &mut buf)
    );

    // invalid member
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark",
            "Members.have.no.dots",
            "/io/killing/spark",
        )
        .build();
    msg.dynheader.serial = Some(1);
    let mut buf = Vec::new();
    assert_eq!(
        Err(crate::Error::Validation(
            crate::params::validation::Error::InvalidMembername
        )),
        marshal(&msg, 0, &mut buf)
    );
}
