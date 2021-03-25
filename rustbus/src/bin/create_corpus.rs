//! This is just a fuzzing helper. It creates some valid dbus messages and dumps them into files.

use rustbus::message_builder::MarshalledMessage;
use rustbus::message_builder::MessageBuilder;
use rustbus::Marshal;

use std::io::Write;

fn main() {
    make_and_dump(
        "./fuzz/corpus/valid_dbus/1.msg",
        "ABCD",
        "asdöflasdölgkjsdfökl",
    );
    make_and_dump(
        "./fuzz/corpus/valid_dbus/2.msg",
        vec!["ABCD", "EFGHI", "JKLMNOP"],
        vec![
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 100, 255, 123, 123, 123, 123, 123, 123,
        ],
    );

    let mut map = std::collections::HashMap::new();
    map.insert("ABCD", (0u8, 100u32));
    map.insert("ABCDEFG", (1u8, 200u32));
    map.insert("X", (2u8, 300u32));
    map.insert("Y", (3u8, 400u32));
    make_and_dump(
        "./fuzz/corpus/valid_dbus/3.msg",
        vec![&map, &map, &map],
        vec![
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 100, 255, 123, 123, 123, 123, 123, 123,
        ],
    );
    make_and_dump(
        "./fuzz/corpus/valid_dbus/4.msg",
        &map,
        vec![
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 100, 255, 123, 123, 123, 123, 123, 123,
        ],
    );
}

fn make_and_dump<P1: Marshal, P2: Marshal>(path: &str, p1: P1, p2: P2) {
    let mut msg = make_message();
    msg.body.push_param2(p1, p2).unwrap();
    dump_message(path, &msg);
}

// Just a default message. The type and stuff will very likely be changed by the fuzzer anyways so I didnt worry about
// creating multiple message headers
fn make_message() -> MarshalledMessage {
    MessageBuilder::new()
        .call("ABCD")
        .on("/A/B/C")
        .with_interface("ABCD.ABCD")
        .at("ABCD.ABCD")
        .build()
}

fn dump_message(path: &str, msg: &MarshalledMessage) {
    let mut hdrbuf = vec![];
    rustbus::wire::marshal::marshal(msg, 0, &mut hdrbuf).unwrap();

    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&hdrbuf).unwrap();
    file.write_all(msg.get_buf()).unwrap();
}
