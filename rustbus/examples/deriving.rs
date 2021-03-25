use rustbus::MessageBuilder;
use rustbus::{Marshal, Signature, Unmarshal};

#[derive(Marshal, Unmarshal, Signature, Default, Debug)]
struct A {
    y: u32,
    x: u64,
    strct: (u8, u8, String),
    raw_data: Vec<u8>,
    sub: SubTypeA,
}

#[derive(Marshal, Unmarshal, Signature, Default, Debug)]
struct SubTypeA {
    x: u8,
    y: u16,
    z: u32,
    w: u64,
    s: String,
}

#[derive(Marshal, Unmarshal, Signature, Default, Debug)]
struct B<'a> {
    y: u32,
    x: u64,
    strct: (u8, u8, &'a str),
    raw_data: &'a [u8],
    sub: SubTypeB<'a>,
}

#[derive(Marshal, Unmarshal, Signature, Default, Debug)]
struct SubTypeB<'a> {
    x: u8,
    y: u16,
    z: u32,
    w: u64,
    s: &'a str,
}

fn main() {
    let a = A {
        y: 0xAAAAAAAA,
        x: 0xBBBBBBBBBBBBBBBB,
        strct: (1, 2, "ABCD".into()),
        raw_data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
        sub: SubTypeA {
            x: 0,
            y: 1,
            z: 3,
            w: 4,
            s: "AA".into(),
        },
    };

    // create a signal with the MessageBuilder API
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark",
            "TestSignal",
            "/io/killing/spark",
        )
        .build();

    // add a parameter to the signal
    sig.body.push_param(&a).unwrap();

    println!("{:#X?}", sig.body);

    println!("{:#X?}", sig.body.parser().get::<A>().unwrap());
    println!("{:#X?}", sig.body.parser().get::<B>().unwrap());
}
