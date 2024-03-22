use std::num::NonZeroU32;

use rustbus::connection::Timeout;
use rustbus::MessageBuilder;
use rustbus::RpcConn;
use rustbus::{Marshal, Signature};

#[derive(Marshal, Signature)]
struct Signal<'a> {
    abcd: u8,
    defgh: u64,
    b: &'a str,
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

    let mut buf = Vec::new();
    for _ in 0..20000000 {
        buf.clear();
        rustbus::wire::marshal::marshal(&sig, NonZeroU32::MIN, &mut buf).unwrap();
    }

    // for _ in 0..50000000 {
    //     let _ = sig.body.parser().get::<(u8, u64, &str)>().unwrap().1;
    // }

    let ctx = con.send_message(&mut sig).unwrap();
    ctx.write_all().unwrap();

    println!("Sent message without a problem!");
}
