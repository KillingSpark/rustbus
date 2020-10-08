use crate::client_conn;
use crate::message_builder::MessageBuilder;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;

const TEST_STRING: &str = "This will be sent over the fd\n";

#[test]
fn test_fd_passing() {
    let mut con1 = client_conn::RpcConn::system_conn(client_conn::Timeout::Infinite).unwrap();
    let mut con2 = client_conn::RpcConn::system_conn(client_conn::Timeout::Infinite).unwrap();
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

    // unmarshal content into params::Param
    let sig = sig.unmarshall_all().unwrap();

    let fd_from_signal = match sig.params[0] {
        crate::params::Param::Base(crate::params::Base::UnixFd(fd)) => fd as RawFd,
        _ => panic!("Did not receive unixfd param"),
    };

    let mut writefile = unsafe { std::fs::File::from_raw_fd(fd_from_signal) };
    writefile.write_all(TEST_STRING.as_bytes()).unwrap();

    let mut line = [0u8; 30];
    readfile.read_exact(&mut line).unwrap();
    assert_eq!(
        String::from_utf8(line.to_vec()).unwrap().as_str(),
        TEST_STRING
    );
}

fn send_fd(con: &mut crate::client_conn::RpcConn, fd: RawFd) -> Result<(), client_conn::Error> {
    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    sig.dynheader.num_fds = Some(1);

    sig.body
        .push_old_param(&crate::params::Param::Base(crate::params::Base::UnixFd(
            fd as u32,
        )))
        .unwrap();

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

#[test]
fn test_fd_marshalling() {
    // not real fds but whatever...
    const TEST_FD1: RawFd = 0xEF;
    const TEST_FD2: RawFd = 0xF0;

    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    sig.body
        .push_old_param(&crate::params::Param::Base(crate::params::Base::UnixFd(
            TEST_FD1 as u32,
        )))
        .unwrap();

    sig.body
        .push_param(crate::wire::marshal::traits::UnixFd(TEST_FD2 as u32))
        .unwrap();

    // assert the correct representation, where fds have been put into the fd array and the index is marshalled in the message
    assert_eq!(sig.body.buf, &[0, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(sig.body.raw_fds, &[TEST_FD1, TEST_FD2]);

    // assert that unmarshalling yields the correct fds
    let mut parser = sig.body.parser();
    let fd1: crate::wire::marshal::traits::UnixFd = parser.get().unwrap();
    let fd2 = parser.get_param().unwrap();

    assert_eq!(crate::wire::marshal::traits::UnixFd(TEST_FD1 as u32), fd1);

    assert!(match fd2 {
        crate::params::Param::Base(crate::params::Base::UnixFd(fd)) => {
            assert_eq!(fd, TEST_FD2 as u32);
            true
        }
        _ => false,
    });
}
