use crate::connection;
use crate::message_builder::MessageBuilder;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::FromRawFd;

const TEST_STRING: &str = "This will be sent over the fd\n";

#[test]
fn test_fd_passing() {
    let mut con1 =
        connection::rpc_conn::RpcConn::system_conn(connection::Timeout::Infinite).unwrap();
    let mut con2 =
        connection::rpc_conn::RpcConn::system_conn(connection::Timeout::Infinite).unwrap();
    con1.send_message(&mut crate::standard_messages::hello())
        .unwrap()
        .write_all()
        .unwrap();
    con2.send_message(&mut crate::standard_messages::hello())
        .unwrap()
        .write_all()
        .unwrap();
    con2.send_message(&mut crate::standard_messages::add_match("type='signal'"))
        .unwrap()
        .write_all()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    let rw = nix::unistd::pipe().unwrap();
    let mut readfile = unsafe { std::fs::File::from_raw_fd(rw.0) };
    send_fd(&mut con1, crate::wire::UnixFd::new(rw.1)).unwrap();

    let sig = loop {
        let signal = con2.wait_signal(connection::Timeout::Infinite).unwrap();
        if signal
            .dynheader
            .interface
            .eq(&Some("io.killing.spark".to_owned()))
            && signal.dynheader.member.eq(&Some("TestSignal".to_owned()))
        {
            break signal;
        }
    };

    let fd_from_signal = sig.body.parser().get::<crate::wire::UnixFd>().unwrap();

    let mut writefile =
        unsafe { std::fs::File::from_raw_fd(fd_from_signal.take_raw_fd().unwrap()) };
    writefile.write_all(TEST_STRING.as_bytes()).unwrap();

    let mut line = [0u8; 30];
    readfile.read_exact(&mut line).unwrap();
    assert_eq!(
        String::from_utf8(line.to_vec()).unwrap().as_str(),
        TEST_STRING
    );
}

fn send_fd(
    con: &mut crate::connection::rpc_conn::RpcConn,
    fd: crate::wire::UnixFd,
) -> Result<(), connection::Error> {
    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();

    sig.dynheader.num_fds = Some(1);

    sig.body.push_param(fd).unwrap();

    con.send_message(&mut sig)?
        .write_all()
        .map_err(crate::connection::ll_conn::force_finish_on_error)
        .unwrap();

    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();
    con.send_message(&mut sig)?
        .write_all()
        .map_err(crate::connection::ll_conn::force_finish_on_error)
        .unwrap();

    Ok(())
}

#[test]
fn test_fd_marshalling() {
    use crate::wire::UnixFd;
    let test_fd1: UnixFd = UnixFd::new(nix::unistd::dup(0).unwrap());
    let test_fd2: UnixFd = UnixFd::new(nix::unistd::dup(1).unwrap());
    let test_fd3: UnixFd = UnixFd::new(nix::unistd::dup(1).unwrap());

    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", "/io/killing/spark")
        .build();

    sig.body
        .push_old_param(&crate::params::Param::Base(crate::params::Base::UnixFd(
            test_fd1.clone(),
        )))
        .unwrap();

    sig.body.push_param(&test_fd2).unwrap();
    sig.body.push_param(&test_fd3).unwrap();

    // assert the correct representation, where fds have been put into the fd array and the index is marshalled in the message
    assert_eq!(sig.body.buf, &[0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0]);
    assert_ne!(&sig.body.raw_fds[0], &test_fd1);
    assert_ne!(&sig.body.raw_fds[1], &test_fd2);
    assert_ne!(&sig.body.raw_fds[2], &test_fd3);

    // assert that unmarshalling yields the correct fds
    let mut parser = sig.body.parser();
    let _fd1: crate::wire::UnixFd = parser.get().unwrap();
    // get _fd2
    assert!(match parser.get_param().unwrap() {
        crate::params::Param::Base(crate::params::Base::UnixFd(_fd)) => {
            true
        }
        _ => false,
    });
    let _fd3: crate::wire::UnixFd = parser.get().unwrap();

    // Take all fds back to prevent accidental closing of actual FDs
    test_fd1.take_raw_fd().unwrap();
    test_fd2.take_raw_fd().unwrap();
    test_fd3.take_raw_fd().unwrap();
}
