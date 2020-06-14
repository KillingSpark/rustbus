use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustbus::params::Container;
use rustbus::params::DictMap;
use rustbus::params::Param;
use rustbus::wire::marshal::marshal;
use rustbus::wire::unmarshal::unmarshal_dynamic_header;
use rustbus::wire::unmarshal::unmarshal_header;
use rustbus::wire::unmarshal::unmarshal_next_message;

fn marsh(msg: &rustbus::message_builder::MarshalledMessage, buf: &mut Vec<u8>) {
    marshal(msg, rustbus::ByteOrder::LittleEndian, &[], buf).unwrap();
}

fn unmarshal(buf: &[u8]) {
    let (hdrbytes, header) = unmarshal_header(&buf, 0).unwrap();
    let (dynhdrbytes, dynheader) = unmarshal_dynamic_header(&header, &buf, hdrbytes).unwrap();
    let (_, _unmarshed_msg) =
        unmarshal_next_message(&header, dynheader, &buf, hdrbytes + dynhdrbytes).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut params: Vec<Param> = Vec::new();

    let mut dict = DictMap::new();
    dict.insert("A".into(), 1234567i32.into());
    dict.insert("B".into(), 1234567i32.into());
    dict.insert("C".into(), 1234567i32.into());
    dict.insert("D".into(), 1234567i32.into());
    dict.insert("E".into(), 1234567i32.into());

    use std::convert::TryFrom;
    let dict: Param = Container::try_from(dict).unwrap().into();

    let array: Param = Container::make_array(
        "s",
        &mut (0..1024).map(|i| format!("{}{}{}{}{}{}{}{}{}", i, i, i, i, i, i, i, i, i)),
    )
    .unwrap()
    .into();

    let ref_array: &[Param] = &["ABCD".into()];
    let ref_array: Param = Container::make_array_ref("s", ref_array).unwrap().into();

    for _ in 0..10 {
        params.push("TesttestTesttest".into());
        params.push(0xFFFFFFFFFFFFFFFFu64.into());
        params.push(
            Container::Struct(vec![
                0xFFFFFFFFFFFFFFFFu64.into(),
                "TesttestTesttest".into(),
            ])
            .into(),
        );
        params.push(dict.clone());
        params.push(array.clone());
        params.push(ref_array.clone());
    }

    let mut msg = rustbus::message_builder::MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    msg.body.push_old_params(&params).unwrap();
    msg.dynheader.serial = Some(1);
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
