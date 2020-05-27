use crate::params::Base;
use crate::params::Param;
use crate::wire::marshal::marshal;
use crate::wire::unmarshal::unmarshal_header;
use crate::wire::unmarshal::unmarshal_next_message;

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
        unmarshal_next_message(&header, &buf, crate::wire::unmarshal::HEADER_LEN).unwrap();

    assert_eq!(msg.params, unmarshed_msg.params);
}

// this tests that invalid inputs do not panic but return errors
#[test]
fn test_invalid_stuff() {
    // invalid signature
    let mut params: Vec<Param> = Vec::new();
    params.push(Base::Signature("((((((((}}}}}}}".into()).into());
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
    assert_eq!(
        Err(crate::message::Error::InvalidSignature(
            crate::signature::Error::InvalidSignature
        )),
        marshal(&msg, crate::message::ByteOrder::LittleEndian, &[], &mut buf)
    );
    // invalid objectpath
    let mut params: Vec<Param> = Vec::new();
    params.push(Base::ObjectPath("invalid/object/path".into()).into());
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
    assert_eq!(
        Err(crate::message::Error::InvalidObjectPath),
        marshal(&msg, crate::message::ByteOrder::LittleEndian, &[], &mut buf)
    );

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

#[test]
fn verify_base_marshalling() {
    let mut buf = vec![];

    let param = crate::params::Base::Uint32(32);
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[32, 0, 0, 0]);
    buf.clear();

    let param = crate::params::Base::Uint64(32u64 + (64u64 << (7 * 8)));
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[32, 0, 0, 0, 0, 0, 0, 64]);
    buf.clear();

    let param = crate::params::Base::Uint16(32 << 8);
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[0, 32]);
    buf.clear();

    let param = crate::params::Base::Byte(32);
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[32]);
    buf.clear();

    let param = crate::params::Base::Boolean(true);
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[1, 0, 0, 0]);
    buf.clear();
    let param = crate::params::Base::Boolean(false);
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[0, 0, 0, 0]);
    buf.clear();

    let param = crate::params::Base::Signature("(vvv)aa{ii}".to_owned());
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(
        buf,
        &[11, b'(', b'v', b'v', b'v', b')', b'a', b'a', b'{', b'i', b'i', b'}', b'\0']
    );
    buf.clear();

    let param = crate::params::Base::String("(vvv)aa{ii}".to_owned());
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(
        buf,
        &[11, 0, 0, 0, b'(', b'v', b'v', b'v', b')', b'a', b'a', b'{', b'i', b'i', b'}', b'\0']
    );
    buf.clear();

    let param = crate::params::Base::ObjectPath("/path".to_owned());
    crate::wire::marshal_base::marshal_base_param(
        crate::message::ByteOrder::LittleEndian,
        &param,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[5, 0, 0, 0, b'/', b'p', b'a', b't', b'h', b'\0']);
    buf.clear();
}

#[test]
fn verify_array_marshalling() {
    let mut buf = vec![];

    // array with one u32
    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let arr = crate::params::Container::make_array("u", vec![param].into_iter()).unwrap();

    crate::wire::marshal_container::marshal_container_param(
        &arr,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[4, 0, 0, 0, 32, 0, 0, 0]);
    buf.clear();

    // array with two u32
    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let arr =
        crate::params::Container::make_array("u", vec![param.clone(), param.clone()].into_iter())
            .unwrap();

    crate::wire::marshal_container::marshal_container_param(
        &arr,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    assert_eq!(buf, &[8, 0, 0, 0, 32, 0, 0, 0, 32, 0, 0, 0]);
    buf.clear();

    // array with two u64
    let param = crate::params::Param::Base(crate::params::Base::Uint64(32));
    let arr =
        crate::params::Container::make_array("t", vec![param.clone(), param.clone()].into_iter())
            .unwrap();

    crate::wire::marshal_container::marshal_container_param(
        &arr,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    assert_eq!(
        buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the u64 elements
        // Also note that the length is 16. The padding is not included in the encoded length value.
        &[16, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0]
    );
    buf.clear();

    // array with two structs
    let param = crate::params::Param::Base(crate::params::Base::Uint64(32));
    let param2 = crate::params::Param::Base(crate::params::Base::Uint64(64));
    let strct = crate::params::Container::make_struct(vec![param.clone(), param2.clone()]);
    let arr = crate::params::Container::make_array(
        "(tt)",
        vec![strct.clone(), strct.clone()].into_iter(),
    )
    .unwrap();

    crate::wire::marshal_container::marshal_container_param(
        &arr,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    assert_eq!(
        buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the struct elements
        // Also note that the length is 32. The padding is not included in the encoded length value.
        vec![
            32, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0,
            0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0
        ]
    );
    buf.clear();
}

#[test]
fn verify_dict_marshalling() {
    let mut buf = vec![];

    let mut dict: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
    dict.insert(64u64, 32u32);

    let dict = crate::params::Container::make_dict("t", "u", dict.into_iter()).unwrap();

    crate::wire::marshal_container::marshal_container_param(
        &dict,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    assert_eq!(
        buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the dict-entry
        // Also note that the length is 12. The padding is not included in the encoded length value.
        &[12, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0]
    );
    buf.clear();

    let mut dict: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
    dict.insert(32u32, 64u64);

    let dict = crate::params::Container::make_dict("u", "t", dict.into_iter()).unwrap();

    crate::wire::marshal_container::marshal_container_param(
        &dict,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();
    assert_eq!(
        buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the dict-entry
        // Also note that the length is 16. The padding is not included in the encoded length value.
        &[16, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,]
    );
    buf.clear();
}

#[test]
fn verify_variant_marshalling() {
    let mut buf = vec![];

    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let v = crate::params::Container::make_variant(param);

    crate::wire::marshal_container::marshal_container_param(
        &v,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();

    // signature ++ padding ++ 32u32
    assert_eq!(buf, &[1, b'u', 0, 0, 32, 0, 0, 0]);
    buf.clear();

    let param = crate::params::Param::Base(crate::params::Base::Uint64(32));
    let param2 = crate::params::Param::Base(crate::params::Base::Uint64(64));
    let strct = crate::params::Container::make_struct(vec![param, param2]);
    let v = crate::params::Container::make_variant(strct);

    crate::wire::marshal_container::marshal_container_param(
        &v,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();

    // signature ++ padding ++ 32u32 ++ 64u64
    assert_eq!(
        buf,
        &[4, b'(', b't', b't', b')', 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0]
    );
    buf.clear();

    let param = crate::params::Param::Base(crate::params::Base::Byte(32));
    let v = crate::params::Container::make_variant(param);

    crate::wire::marshal_container::marshal_container_param(
        &v,
        crate::message::ByteOrder::LittleEndian,
        &mut buf,
    )
    .unwrap();

    // signature ++ padding ++ 32u8
    assert_eq!(buf, &[1, b'y', 0, 32]);
    buf.clear();
}
