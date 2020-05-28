use crate::client_conn;
use crate::message_builder::MessageBuilder;
use std::io::Write;
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;

const TEST_STRING: &str = "This will be sent over the fd\n";

#[test]
fn test_fd_passing() {
    let mut con1 = client_conn::RpcConn::session_conn(client_conn::Timeout::Infinite).unwrap();
    let mut con2 = client_conn::RpcConn::session_conn(client_conn::Timeout::Infinite).unwrap();
    con1.send_message(
        &mut crate::standard_messages::hello(),
        client_conn::Timeout::Infinite,
    )
    .unwrap();
    con2.send_message(
        &mut crate::standard_messages::hello(),
        client_conn::Timeout::Infinite,
    )
    .unwrap();
    con2.send_message(
        &mut crate::standard_messages::add_match("type='signal'".into()),
        client_conn::Timeout::Infinite,
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    let rw = nix::unistd::pipe().unwrap();
    let mut readfile = unsafe { std::fs::File::from_raw_fd(rw.0) };
    send_fd(&mut con1, rw.1).unwrap();

    let sig = loop {
        let signal = con2.wait_signal(client_conn::Timeout::Infinite).unwrap();
        if signal.interface.eq(&Some("io.killing.spark".to_owned())) {
            if signal.member.eq(&Some("TestSignal".to_owned())) {
                break signal;
            }
        }
    };

    let fd_from_signal = sig.raw_fds[0];
    let mut writefile = unsafe { std::fs::File::from_raw_fd(fd_from_signal) };
    writefile.write_all(TEST_STRING.as_bytes()).unwrap();

    let mut line = [0u8;30];
    readfile.read_exact(&mut line).unwrap();
    assert_eq!(String::from_utf8(line.to_vec()).unwrap().as_str(), TEST_STRING);
}

fn send_fd(con: &mut crate::client_conn::RpcConn, fd: RawFd) -> Result<(), client_conn::Error> {
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    sig.raw_fds.push(fd);
    sig.num_fds = Some(1);
    con.send_message(&mut sig, client_conn::Timeout::Infinite)?;

    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();
    con.send_message(&mut sig, client_conn::Timeout::Infinite)?;

    Ok(())
}
