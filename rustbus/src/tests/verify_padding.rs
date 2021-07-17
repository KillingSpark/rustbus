use crate::wire::marshal::base::marshal_base_param;
use crate::wire::marshal::container::marshal_container_param;
use crate::ByteOrder;
use crate::Marshal;

#[test]
fn verify_padding() {
    // always first marshal a byte, then marshal the other type then check that the correct amount of padding was added
    use crate::wire::marshal::MarshalContext;

    let mut fds = Vec::new();
    let mut valid_buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut valid_buf,
        fds: &mut fds,
        byteorder: ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::Uint64(32u64 + (64u64 << (7 * 8)));
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 64]
    );
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32u64 + (64u64 << (7 * 8))).marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 64]
    );
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::Int64(32i64 + (64i64 << (7 * 8)));
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 64]
    );
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32i64 + (64i64 << (7 * 8))).marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 64]
    );
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::Uint32(32 << 8);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 0, 32, 0, 0]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32u32 << 8).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 0, 32, 0, 0,]);
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::Int32(32 << 8);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 0, 32, 0, 0]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32i32 << 8).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 0, 32, 0, 0]);
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::Int16(32 << 8);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 32]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32i16 << 8).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 32]);
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::Uint16(32 << 8);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 32]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32u16 << 8).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 32]);
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::String("A".to_owned());
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 1, 0, 0, 0, b'A', b'\0']);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    "A".marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 1, 0, 0, 0, b'A', b'\0']);
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::ObjectPath("/A/B".to_owned());
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 4, 0, 0, 0, b'/', b'A', b'/', b'B', b'\0']
    );
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    crate::wire::ObjectPath::new("/A/B")
        .unwrap()
        .marshal(ctx)
        .unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 4, 0, 0, 0, b'/', b'A', b'/', b'B', b'\0']
    );
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Base::ObjectPath("/A/B".to_owned());
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 4, 0, 0, 0, b'/', b'A', b'/', b'B', b'\0']
    );
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    crate::wire::ObjectPath::new("/A/B")
        .unwrap()
        .marshal(ctx)
        .unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 4, 0, 0, 0, b'/', b'A', b'/', b'B', b'\0']
    );
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let param = crate::params::Param::Base(crate::params::Base::Uint32(32));
    let arr = crate::params::Container::make_array("u", vec![param].into_iter()).unwrap();
    marshal_container_param(&arr, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 4, 0, 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    &[32u32].marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 4, 0, 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();

    0xFFu8.marshal(ctx).unwrap();
    let mut dict: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
    dict.insert(64u64, 32u32);
    let param_dict =
        crate::params::Container::make_dict("t", "u", dict.clone().into_iter()).unwrap();
    marshal_container_param(&param_dict, ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 12, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0]
    );
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    dict.marshal(ctx).unwrap();
    assert_eq!(
        ctx.buf,
        &[0xFF, 0, 0, 0, 12, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0]
    );
    ctx.buf.clear();
}
