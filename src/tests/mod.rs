use crate::params::Base;
use crate::params::Param;
use crate::wire::marshal::marshal;
use crate::wire::unmarshal::unmarshal_header;
use crate::wire::unmarshal::unmarshal_next_message;

mod fdpassing;
mod verify_marshalling;

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
        .build();
    for p in &params {
        msg.body.push_param(p).unwrap();
    }
    msg.serial = Some(1);
    let mut buf = Vec::new();
    marshal(&msg, crate::message::ByteOrder::LittleEndian, &[], &mut buf).unwrap();
    let (_, header) = unmarshal_header(&buf, 0).unwrap();

    let (_, unmarshed_msg) =
        unmarshal_next_message(&header, &buf, crate::wire::unmarshal::HEADER_LEN).unwrap();

    assert_eq!(params, unmarshed_msg.params);
}

// this tests that invalid inputs do not panic but return errors
#[test]
fn test_invalid_stuff() {
    // invalid signature
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    let err = msg
        .body
        .push_param(&Param::Base(Base::Signature("((((((((}}}}}}}".into())));
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        err
    );

    // invalid objectpath
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();
    let err = msg
        .body
        .push_param(&Param::Base(Base::ObjectPath("invalid/object/path".into())));
    assert_eq!(Err(crate::message::Error::InvalidObjectPath), err);

    // invalid interface
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(
            ".......io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();
    msg.serial = Some(1);
    let mut buf = Vec::new();
    assert_eq!(
        Err(crate::message::Error::InvalidInterface),
        marshal(&msg, crate::message::ByteOrder::LittleEndian, &[], &mut buf)
    );

    // invalid member
    let mut msg = crate::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "Members.have.no.dots".into(),
            "/io/killing/spark".into(),
        )
        .build();
    msg.serial = Some(1);
    let mut buf = Vec::new();
    assert_eq!(
        Err(crate::message::Error::InvalidMembername),
        marshal(&msg, crate::message::ByteOrder::LittleEndian, &[], &mut buf)
    );
}

// more specific tests for constraints on strings
#[test]
fn test_objectpath_constraints() {
    let no_beginning_slash = "da/di/du";
    assert_eq!(
        Err(crate::message::Error::InvalidObjectPath),
        crate::params::validate_object_path(no_beginning_slash)
    );
    let empty_element = "/da//du";
    assert_eq!(
        Err(crate::message::Error::InvalidObjectPath),
        crate::params::validate_object_path(empty_element)
    );
    let trailing_slash = "/da/di/du/";
    assert_eq!(
        Err(crate::message::Error::InvalidObjectPath),
        crate::params::validate_object_path(trailing_slash)
    );
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(crate::message::Error::InvalidObjectPath),
        crate::params::validate_object_path(invalid_chars)
    );
    let trailing_slash_on_root = "/";
    assert_eq!(
        Ok(()),
        crate::params::validate_object_path(trailing_slash_on_root)
    );
}
#[test]
fn test_interface_constraints() {
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(crate::message::Error::InvalidInterface),
        crate::params::validate_interface(invalid_chars)
    );
    let leading_digits = "1leading.digits";
    assert_eq!(
        Err(crate::message::Error::InvalidInterface),
        crate::params::validate_interface(leading_digits)
    );
    let too_short = "have_more_than_one_element";
    assert_eq!(
        Err(crate::message::Error::InvalidInterface),
        crate::params::validate_interface(too_short)
    );
    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s.push('.');
        s
    });
    assert_eq!(
        Err(crate::message::Error::InvalidInterface),
        crate::params::validate_interface(&too_long)
    );
}
#[test]
fn test_busname_constraints() {
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(crate::message::Error::InvalidBusname),
        crate::params::validate_busname(invalid_chars)
    );
    let empty = "";
    assert_eq!(
        Err(crate::message::Error::InvalidBusname),
        crate::params::validate_busname(empty)
    );
    let too_short = "have_more_than_one_element";
    assert_eq!(
        Err(crate::message::Error::InvalidBusname),
        crate::params::validate_busname(too_short)
    );

    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s.push('.');
        s
    });
    assert_eq!(
        Err(crate::message::Error::InvalidBusname),
        crate::params::validate_busname(&too_long)
    );
}
#[test]
fn test_membername_constraints() {
    let invalid_chars = "/da$$/di!!/du~~";
    assert_eq!(
        Err(crate::message::Error::InvalidMembername),
        crate::params::validate_membername(invalid_chars)
    );
    let dots = "Shouldnt.have.dots";
    assert_eq!(
        Err(crate::message::Error::InvalidMembername),
        crate::params::validate_membername(dots)
    );
    let empty = "";
    assert_eq!(
        Err(crate::message::Error::InvalidMembername),
        crate::params::validate_membername(empty)
    );

    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s.push('.');
        s
    });
    assert_eq!(
        Err(crate::message::Error::InvalidMembername),
        crate::params::validate_membername(&too_long)
    );
}
#[test]
fn test_signature_constraints() {
    let wrong_parans = "((i)";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let wrong_parans = "(i))";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let wrong_parans = "a{{i}";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let wrong_parans = "a{i}}";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(wrong_parans)
    );
    let array_without_type = "(i)a";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(array_without_type)
    );
    let invalid_chars = "!!§$%&(i)a";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        crate::params::validate_signature(invalid_chars)
    );

    // TODO FIXME this should be an error. Nesting is at maximum 32 for structs and arrays
    let too_deep_nesting = "((((((((((((((((((((((((((((((((()))))))))))))))))))))))))))))))))";
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::NestingTooDeep
        )),
        crate::params::validate_signature(too_deep_nesting)
    );

    let too_long = (0..256).fold(String::new(), |mut s, _| {
        s.push('b');
        s
    });
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::SignatureTooLong
        )),
        crate::params::validate_signature(&too_long)
    );
}
