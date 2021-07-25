use rustbus::connection::Timeout;
use rustbus::MessageBuilder;
use rustbus::RpcConn;
use rustbus::{Marshal, Signature};

#[derive(Marshal)]
struct Signal<'a> {
    abcd: u8,
    defgh: u64,
    b: &'a str,
}

impl<'a> Signature for Signal<'a> {
    unsafe fn valid_slice(_bo: rustbus::ByteOrder) -> bool {
        false
    }

    fn sig_str(s_buf: &mut rustbus::wire::marshal::traits::SignatureBuffer) {
        s_buf.push_static("(yts)")
    }

    fn signature() -> rustbus::signature::Type {
        rustbus::signature::Type::Container(rustbus::signature::Container::Struct(
            rustbus::signature::StructTypes::new(vec![
                rustbus::signature::Type::Base(rustbus::signature::Base::Byte),
                rustbus::signature::Type::Base(rustbus::signature::Base::Uint64),
                rustbus::signature::Type::Base(rustbus::signature::Base::String),
            ])
            .unwrap(),
        ))
    }

    fn alignment() -> usize {
        8
    }
}

fn main() {
    let mut con = RpcConn::session_conn(Timeout::Infinite).unwrap();

    let mut sig = MessageBuilder::new()
        .signal("io.killingspark", "Signal", "/io/killingspark/Signaler")
        .build();

    sig.body
        .push_param(Signal {
            abcd: 100,
            defgh: 200,
            b: "ABCDEFGH",
        })
        .unwrap();

    for _ in 0..50000000 {
        let _ = sig.body.parser().get::<(u8, u64, &str)>().unwrap().1;
    }

    let ctx = con.send_message(&mut sig).unwrap();
    ctx.write_all().unwrap();

    println!("Sent message without a problem!");
}
