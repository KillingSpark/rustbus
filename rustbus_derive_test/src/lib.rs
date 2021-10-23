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
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
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

#[test]
pub fn test_enum_derive() {
    use rustbus::wire::unmarshal::traits::Variant;
    use rustbus::MessageBuilder;
    use rustbus_derive::{Marshal, Signature, Unmarshal};

    #[derive(Marshal, Unmarshal, Signature, PartialEq, Eq, Debug)]
    enum Variant1<'a> {
        A(String),
        B(&'a str, String),
        C {
            c1: String,
            c2: String,
            c3: u64,
            c4: (u8, i32, i64, bool),
        },
    }

    #[derive(Marshal, Unmarshal, Signature, PartialEq, Eq, Debug)]
    enum Variant2 {
        A(u64),
    }

    let v1 = Variant1::A("ABCD".into());
    let v2 = Variant1::B("ABCD".into(), "EFGH".into());
    let v3 = Variant1::C {
        c1: "ABCD".into(),
        c2: "EFGH".into(),
        c3: 100,
        c4: (000, 100, 200, false),
    };

    // create a signal with the MessageBuilder API
    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();

    // add a parameter to the signal
    sig.body.push_param(&v1).unwrap();
    sig.body.push_param(&v2).unwrap();
    sig.body.push_param(&v3).unwrap();

    // use generic Variant type to parse the variants by hand
    let (m1, m2, m3) = sig
        .body
        .parser()
        .get3::<Variant, Variant, Variant>()
        .unwrap();

    let v1_2 = Variant1::A(m1.get().unwrap());

    let v2_2 = m2.get::<(String, String)>().unwrap();
    let v2_2 = Variant1::B(&v2_2.0, v2_2.1);

    let v3_2 = m3
        .get::<(String, String, u64, (u8, i32, i64, bool))>()
        .unwrap();
    let v3_2 = Variant1::C {
        c1: v3_2.0,
        c2: v3_2.1,
        c3: v3_2.2,
        c4: v3_2.3,
    };

    assert_eq!(v1, v1_2);
    assert_eq!(v2, v2_2);
    assert_eq!(v3, v3_2);

    // use the derived unmarshal impl to get the variants
    let (v1_3, v2_3, v3_3) = sig
        .body
        .parser()
        .get3::<Variant1, Variant1, Variant1>()
        .unwrap();

    assert_eq!(v1, v1_3);
    assert_eq!(v2, v2_3);
    assert_eq!(v3, v3_3);

    let err = sig.body.parser().get::<Variant2>();
    assert_eq!(
        Err(::rustbus::wire::unmarshal::Error::NoMatchingVariantFound),
        err
    );
}
