extern crate rustbus;
use rustbus::message_builder::MessageBuilder;
use rustbus::standard_messages;

use std::io::Write;
use std::os::unix::io::FromRawFd;

fn main() {
    if std::env::args()
        .collect::<Vec<_>>()
        .contains(&"send".to_owned())
    {
        send_fd();
    } else {
        let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
        let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
        let mut con = rustbus::client_conn::RpcConn::new(con);
        con.send_message(rustbus::standard_messages::hello())
            .unwrap();

        con.send_message(standard_messages::add_match("type='signal'".into()))
            .unwrap();

        let sig = loop {
            let signal = con.wait_signal().unwrap();
            println!("Got signal: {:?}", signal);
            if signal.interface.eq(&Some("io.killing.spark".to_owned())) {
                if signal.member.eq(&Some("TestSignal".to_owned())) {
                    break signal;
                }
            }
        };

        println!("Got signal: {:?}", sig);
        let mut file = unsafe { std::fs::File::from_raw_fd(sig.raw_fds[0]) };
        file.write_all(b"This is a line\n").unwrap();
    }
}

fn send_fd() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let mut con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
    con.send_message(rustbus::standard_messages::hello())
        .unwrap();
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    sig.raw_fds.push(0);
    sig.num_fds = Some(1);
    con.send_message(sig).unwrap();

    let sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();
    con.send_message(sig).unwrap();

    println!("Printing stuff fromn stdin");
    let mut line = String::new();
    loop {
        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        println!("Line: {}", line);
    }
}
