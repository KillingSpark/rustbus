use std::num::NonZeroU32;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustbus::params::Container;
use rustbus::params::DictMap;
use rustbus::params::Param;
use rustbus::wire::marshal::marshal;
use rustbus::wire::unmarshal::unmarshal_dynamic_header;
use rustbus::wire::unmarshal::unmarshal_header;
use rustbus::wire::unmarshal::unmarshal_next_message;
use rustbus::wire::unmarshal_context::Cursor;

fn marsh(msg: &rustbus::message_builder::MarshalledMessage, buf: &mut Vec<u8>) {
    marshal(msg, NonZeroU32::MIN, buf).unwrap();
}

fn unmarshal(buf: &[u8]) {
    let mut cursor = Cursor::new(buf);
    let header = unmarshal_header(&mut cursor).unwrap();
    let dynheader = unmarshal_dynamic_header(&header, &mut cursor).unwrap();
    let _unmarshed_msg =
        unmarshal_next_message(&header, dynheader, buf.to_vec(), cursor.consumed(), vec![])
            .unwrap();
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

    let mut buf = Vec::new();
    c.bench_function("marshal", |b| {
        b.iter(|| {
            let mut msg = rustbus::message_builder::MessageBuilder::new()
                .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
                .build();
            msg.body.push_old_params(&params).unwrap();
            msg.dynheader.serial = Some(NonZeroU32::MIN);
            buf.clear();
            marsh(black_box(&msg), &mut buf)
        })
    });

    let mut msg = rustbus::message_builder::MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();
    msg.body.push_old_params(&params).unwrap();
    msg.dynheader.serial = Some(NonZeroU32::MIN);
    buf.clear();
    marsh(&msg, &mut buf);
    buf.extend_from_slice(msg.get_buf());
    c.bench_function("unmarshal", |b| b.iter(|| unmarshal(black_box(&buf))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
