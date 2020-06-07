use rustbus::{
    client_conn::Timeout, get_session_bus_path, standard_messages, Conn, MessageBuilder, RpcConn,
};

use std::io::Write;
use std::os::unix::io::FromRawFd;

fn main() -> Result<(), rustbus::client_conn::Error> {
    if std::env::args()
        .collect::<Vec<_>>()
        .contains(&"send".to_owned())
    {
        send_fd()?;
    } else {
        let session_path = get_session_bus_path()?;
        let con = Conn::connect_to_bus(session_path, true)?;
        let mut con = RpcConn::new(con);
        con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;

        con.send_message(
            &mut standard_messages::add_match("type='signal'".into()),
            Timeout::Infinite,
        )?;

        let sig = loop {
            let signal = con.wait_signal(Timeout::Infinite)?;
            println!("Got signal: {:?}", signal);
            if signal
                .dynheader
                .interface
                .eq(&Some("io.killing.spark".to_owned()))
            {
                if signal.dynheader.member.eq(&Some("TestSignal".to_owned())) {
                    break signal;
                }
            }
        };

        println!("Got signal: {:?}", sig);
        let mut file = unsafe { std::fs::File::from_raw_fd(sig.raw_fds[0]) };
        file.write_all(b"This is a line\n")?;
    }

    Ok(())
}

fn send_fd() -> Result<(), rustbus::client_conn::Error> {
    let session_path = rustbus::client_conn::get_session_bus_path()?;
    let mut con = rustbus::client_conn::Conn::connect_to_bus(session_path, true)?;
    con.send_message(&mut rustbus::standard_messages::hello(), Timeout::Infinite)?;
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    sig.raw_fds.push(0);
    sig.dynheader.num_fds = Some(1);
    con.send_message(&mut sig, Timeout::Infinite)?;

    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();
    con.send_message(&mut sig, Timeout::Infinite)?;

    println!("Printing stuff fromn stdin");
    let mut line = String::new();
    loop {
        line.clear();
        std::io::stdin().read_line(&mut line)?;
        println!("Line: {}", line);
    }
}
