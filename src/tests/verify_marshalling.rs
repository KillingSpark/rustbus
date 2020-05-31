use crate::Marshal;

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
    32u32
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    (32u64 + (64u64 << (7 * 8)))
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    (32u16 << 8)
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    32u8.marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    true.marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    false
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
        .unwrap();
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
    crate::wire::marshal_trait::Signature::new("(vvv)aa{ii}")
        .unwrap()
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    "(vvv)aa{ii}"
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
    crate::wire::marshal_trait::ObjectPath::new("/path")
        .unwrap()
        .marshal(crate::message::ByteOrder::LittleEndian, &mut buf)
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
