use rustbus::{
    connection::Timeout, get_session_bus_path, standard_messages, DuplexConn, MessageBuilder,
    RpcConn,
};

use std::io::Write;
use std::os::unix::io::FromRawFd;

fn main() -> Result<(), rustbus::connection::Error> {
    if std::env::args()
        .collect::<Vec<_>>()
        .contains(&"send".to_owned())
    {
        send_fd()?;
    } else {
        let session_path = get_session_bus_path()?;
        let con = DuplexConn::connect_to_bus(session_path, true)?;
        let mut con = RpcConn::new(con);
        con.send_message(&mut standard_messages::hello())?
            .write_all()
            .unwrap();

        con.send_message(&mut standard_messages::add_match("type='signal'".into()))?
            .write_all()
            .unwrap();

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
        let fd: rustbus::wire::UnixFd = sig.body.parser().get().unwrap();

        let mut file = unsafe { std::fs::File::from_raw_fd(fd.take_raw_fd().unwrap()) };
        file.write_all(
            format!(
                "This is a line from process with pid: {}\n",
                std::process::id()
            )
            .as_bytes(),
        )?;
    }

    Ok(())
}

fn send_fd() -> Result<(), rustbus::connection::Error> {
    let session_path = rustbus::connection::get_session_bus_path()?;
    let mut con = rustbus::DuplexConn::connect_to_bus(session_path, true)?;
    con.send_hello(Timeout::Infinite).unwrap();
    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();

    use std::os::unix::io::AsRawFd;
    let stdin_fd = std::io::stdin();
    sig.body.push_param((&stdin_fd) as &dyn AsRawFd).unwrap();
    sig.dynheader.num_fds = Some(1);
    con.send.send_message(&mut sig)?.write_all().unwrap();

    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();
    con.send.send_message(&mut sig)?.write_all().unwrap();

    println!("Printing stuff from stdin. The following is input from the other process!");
    let mut line = String::new();
    loop {
        line.clear();
        std::io::stdin().read_line(&mut line)?;
        println!("Line: {}", line);
    }
}
