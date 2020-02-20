use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustbus::marshal::marshal;
use rustbus::message::Container;
use rustbus::message::DictMap;
use rustbus::message::Param;
use rustbus::unmarshal::unmarshal_header;
use rustbus::unmarshal::unmarshal_next_message;

fn marsh(msg: &rustbus::Message, buf: &mut Vec<u8>) {
    marshal(msg, rustbus::message::ByteOrder::LittleEndian, &[], buf).unwrap();
}

fn unmarshal(buf: &[u8]) {
    let (_, header) = unmarshal_header(&buf, 0).unwrap();
    let (_, _unmarshed_msg) =
        unmarshal_next_message(&header, &buf, rustbus::unmarshal::HEADER_LEN).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut params: Vec<Param> = Vec::new();

    let mut dict = DictMap::new();
    dict.insert("A".to_owned().into(), 1234567i32.into());
    dict.insert("B".to_owned().into(), 1234567i32.into());
    dict.insert("C".to_owned().into(), 1234567i32.into());
    dict.insert("D".to_owned().into(), 1234567i32.into());
    dict.insert("E".to_owned().into(), 1234567i32.into());

    use std::convert::TryFrom;
    let dict: Param = Container::try_from(dict).unwrap().into();

    let array: Param = Container::try_from(vec![
        0xFFFFFFFFFFFFFFFFu64.into(),
        0xFFFFFFFFFFFFFFFFu64.into(),
        0xFFFFFFFFFFFFFFFFu64.into(),
        0xFFFFFFFFFFFFFFFFu64.into(),
        0xFFFFFFFFFFFFFFFFu64.into(),
    ])
    .unwrap()
    .into();

    for _ in 0..10 {
        params.push("TesttestTesttest".to_owned().into());
        params.push(0xFFFFFFFFFFFFFFFFu64.into());
        params.push(
            Container::Struct(vec![
                0xFFFFFFFFFFFFFFFFu64.into(),
                "TesttestTesttest".to_owned().into(),
            ])
            .into(),
        );
        params.push(dict.clone());
        params.push(array.clone());
    }

    let mut msg = rustbus::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .with_params(params)
        .build();
    msg.serial = Some(1);
    let mut buf = Vec::new();
    c.bench_function("marshal", |b| {
        b.iter(|| {
            buf.clear();
            marsh(black_box(&msg), &mut buf)
        })
    });

    buf.clear();
    marsh(&msg, &mut buf);
    c.bench_function("unmarshal", |b| b.iter(|| unmarshal(black_box(&buf))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
