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
    let param = crate::params::Base::Uint32(32);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    32u32.marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 0, 32, 0, 0, 0]);
    ctx.buf.clear();

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
    let param = crate::params::Base::Uint16(32 << 8);
    marshal_base_param(&param, ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 32]);
    ctx.buf.clear();
    0xFFu8.marshal(ctx).unwrap();
    (32u16 << 8).marshal(ctx).unwrap();
    assert_eq!(ctx.buf, &[0xFF, 0, 0, 32]);
    ctx.buf.clear();
}
