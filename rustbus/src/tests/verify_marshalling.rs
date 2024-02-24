use crate::wire::marshal::base::marshal_base_param;
use crate::wire::marshal::container::marshal_container_param;
use crate::ByteOrder;
use crate::Marshal;

#[test]
fn verify_base_marshalling() {
    use crate::wire::marshal::MarshalContext;

    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    let param = crate::params::Base::Uint32(32);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[32, 0, 0, 0]);
    ctx.buf.clear();
    32u32.marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[32, 0, 0, 0]);
    ctx.buf.clear();

    let param = crate::params::Base::Uint64(32u64 + (64u64 << (7 * 8)));
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[32, 0, 0, 0, 0, 0, 0, 64]);
    ctx.buf.clear();
    (32u64 + (64u64 << (7 * 8))).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[32, 0, 0, 0, 0, 0, 0, 64]);
    ctx.buf.clear();

    let param = crate::params::Base::Uint16(32 << 8);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0, 32]);
    ctx.buf.clear();
    (32u16 << 8).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0, 32]);
    ctx.buf.clear();

    let param = crate::params::Base::Byte(32);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[32]);
    ctx.buf.clear();
    32u8.marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[32]);
    ctx.buf.clear();

    let param = crate::params::Base::Boolean(true);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[1, 0, 0, 0]);
    ctx.buf.clear();
    true.marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[1, 0, 0, 0]);
    ctx.buf.clear();
    let param = crate::params::Base::Boolean(false);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0, 0, 0, 0]);
    ctx.buf.clear();
    false.marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0, 0, 0, 0]);
    ctx.buf.clear();

    let param = crate::params::Base::Signature("(vvv)aa{ii}".to_owned());
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[11, b'(', b'v', b'v', b'v', b')', b'a', b'a', b'{', b'i', b'i', b'}', b'\0']
    );
    ctx.buf.clear();
    crate::wire::SignatureWrapper::new("(vvv)aa{ii}")
        .unwrap()
        .marshal(ctx)
        .unwrap();
    assert_eq!(
        ctx.buf,
        &[11, b'(', b'v', b'v', b'v', b')', b'a', b'a', b'{', b'i', b'i', b'}', b'\0']
    );
    ctx.buf.clear();

    let param = crate::params::Base::String("(vvv)aa{ii}".to_owned());
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[11, 0, 0, 0, b'(', b'v', b'v', b'v', b')', b'a', b'a', b'{', b'i', b'i', b'}', b'\0']
    );
    ctx.buf.clear();
    "(vvv)aa{ii}".marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[11, 0, 0, 0, b'(', b'v', b'v', b'v', b')', b'a', b'a', b'{', b'i', b'i', b'}', b'\0']
    );
    ctx.buf.clear();

    let param = crate::params::Base::ObjectPath("/path".to_owned());
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[5, 0, 0, 0, b'/', b'p', b'a', b't', b'h', b'\0']);
    ctx.buf.clear();
    crate::wire::ObjectPath::new("/path")
        .unwrap()
        .marshal(ctx)
        .unwrap();
    assert_eq!(ctx.buf, &[5, 0, 0, 0, b'/', b'p', b'a', b't', b'h', b'\0']);
    ctx.buf.clear();
}

#[test]
fn verify_array_marshalling() {
    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // array with one u32
    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let arr = crate::params::Container::make_array("u", vec![param].into_iter()).unwrap();

    marshal_container_param(&arr, ctx).unwrap();
    assert_eq!(ctx.buf, &[4, 0, 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();

    // array with two u32
    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let arr =
        crate::params::Container::make_array("u", vec![param.clone(), param.clone()].into_iter())
            .unwrap();

    marshal_container_param(&arr, ctx).unwrap();
    assert_eq!(ctx.buf, &[8, 0, 0, 0, 32, 0, 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();

    // array with two u64
    let param = crate::params::Param::Base(crate::params::Base::Uint64(32));
    let arr =
        crate::params::Container::make_array("t", vec![param.clone(), param.clone()].into_iter())
            .unwrap();

    marshal_container_param(&arr, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the u64 elements
        // Also note that the length is 16. The padding is not included in the encoded length value.
        &[16, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0]
    );
    ctx.buf.clear();

    // array with two structs
    let param = crate::params::Param::Base(crate::params::Base::Uint64(32));
    let param2 = crate::params::Param::Base(crate::params::Base::Uint64(64));
    let strct = crate::params::Container::make_struct(vec![param.clone(), param2.clone()]);
    let arr = crate::params::Container::make_array(
        "(tt)",
        vec![strct.clone(), strct.clone()].into_iter(),
    )
    .unwrap();

    marshal_container_param(&arr, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the struct elements
        // Also note that the length is 32. The padding is not included in the encoded length value.
        &vec![
            32, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0,
            0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0
        ]
    );
    ctx.buf.clear();

    use crate::wire::marshal::MarshalContext;

    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;
    [(32u64, 64u64), (32u64, 64u64)][..].marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the struct elements
        // Also note that the length is 32. The padding is not included in the encoded length value.
        &vec![
            32, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0,
            0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0
        ]
    );
    ctx.buf.clear();
}

#[test]
fn verify_dict_marshalling() {
    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    let mut dict: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
    dict.insert(64u64, 32u32);

    let dict = crate::params::Container::make_dict("t", "u", dict.into_iter()).unwrap();

    marshal_container_param(&dict, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the dict-entry
        // Also note that the length is 12. The padding is not included in the encoded length value.
        &[12, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0]
    );
    ctx.buf.clear();

    let mut map: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
    map.insert(32u32, 64u64);

    let dict = crate::params::Container::make_dict("u", "t", map.clone().into_iter()).unwrap();

    marshal_container_param(&dict, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the dict-entry
        // Also note that the length is 16. The padding is not included in the encoded length value.
        &[16, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,]
    );
    ctx.buf.clear();

    use crate::wire::marshal::MarshalContext;

    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;
    map.marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        // Note the longer \0 chain after the length. This is the needed padding after the u32 length and the dict-entry
        // Also note that the length is 16. The padding is not included in the encoded length value.
        &[16, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,]
    );
    ctx.buf.clear();
}

#[test]
fn verify_variant_marshalling() {
    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = crate::wire::marshal::MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let v = crate::params::Container::make_variant(param);

    marshal_container_param(&v, ctx).unwrap();

    // signature ++ padding ++ 32u32
    assert_eq!(ctx.buf, &[1, b'u', 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();

    let param = crate::params::Param::Base(crate::params::Base::Uint64(32));
    let param2 = crate::params::Param::Base(crate::params::Base::Uint64(64));
    let strct = crate::params::Container::make_struct(vec![param, param2]);
    let v = crate::params::Container::make_variant(strct);

    marshal_container_param(&v, ctx).unwrap();

    // signature ++ padding ++ 32u32 ++ 64u64
    assert_eq!(
        ctx.buf,
        &[4, b'(', b't', b't', b')', 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0]
    );
    ctx.buf.clear();

    let param = crate::params::Param::Base(crate::params::Base::Byte(32));
    let v = crate::params::Container::make_variant(param);

    marshal_container_param(&v, ctx).unwrap();

    // signature ++ padding ++ 32u8
    assert_eq!(ctx.buf, &[1, b'y', 0, 32]);
    ctx.buf.clear();

    let param = crate::params::Param::Base(crate::params::Base::Byte(16));
    let v = crate::params::Variant {
        sig: crate::signature::Type::Base(crate::signature::Base::Byte),
        value: param,
    };

    v.marshal(ctx).unwrap();

    // signature ++ padding ++ 16u8
    assert_eq!(ctx.buf, &[1, b'y', 0, 16]);
    ctx.buf.clear();
}
