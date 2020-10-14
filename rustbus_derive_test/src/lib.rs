#[test]
fn test_derive() {
    use rustbus::message_builder::MessageBuilder;
    use rustbus_derive::{Marshal, Signature, Unmarshal};
    #[derive(Marshal, Unmarshal, Signature, Default, Debug, Eq, PartialEq)]
    struct A {
        y: u32,
        x: u64,
        strct: (u8, u8, String),
        raw_data: Vec<u8>,
        sub: SubTypeA,
    }

    #[derive(Marshal, Unmarshal, Signature, Default, Debug, Eq, PartialEq)]
    struct SubTypeA {
        x: u8,
        y: u16,
        z: u32,
        w: u64,
        s: String,
    }

    #[derive(Marshal, Unmarshal, Signature, Default, Debug, Eq, PartialEq)]
    struct B<'a> {
        y: u32,
        x: u64,
        strct: (u8, u8, &'a str),
        raw_data: &'a [u8],
        sub: SubTypeB<'a>,
    }

    #[derive(Marshal, Unmarshal, Signature, Default, Debug, Eq, PartialEq)]
    struct SubTypeB<'a> {
        x: u8,
        y: u16,
        z: u32,
        w: u64,
        s: &'a str,
    }

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
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    // add a parameter to the signal
    sig.body.push_param(&a).unwrap();

    assert_eq!(a, sig.body.parser().get::<A>().unwrap());

    let b = B {
        x: a.x,
        y: a.y,
        raw_data: &a.raw_data,
        strct: (a.strct.0, a.strct.1, &a.strct.2),
        sub: SubTypeB {
            x: a.sub.x,
            y: a.sub.y,
            z: a.sub.z,
            w: a.sub.w,
            s: &a.sub.s,
        },
    };
    assert_eq!(b, sig.body.parser().get::<B>().unwrap());
}
