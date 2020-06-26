use crate::client_conn::RpcConn;
use crate::client_conn::Timeout;
use crate::standard_messages;

// This tests that messages sent by dbus-send are understood

#[test]
fn test_dbus_send_comp() -> Result<(), crate::client_conn::Error> {
    let mut rpc_con = RpcConn::session_conn(Timeout::Infinite).unwrap();

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        crate::message_builder::MessageType::Call => false,
        crate::message_builder::MessageType::Invalid => false,
        crate::message_builder::MessageType::Error => true,
        crate::message_builder::MessageType::Reply => true,
        crate::message_builder::MessageType::Signal => msg
            .dynheader
            .interface
            .eq(&Some("io.killing.spark.dbustest".to_owned())),
    }));

    let hello_serial = rpc_con.send_message(
        &mut standard_messages::hello(),
        Timeout::Duration(std::time::Duration::from_millis(10)),
    )?;
    let _msg = rpc_con.wait_response(
        hello_serial,
        Timeout::Duration(std::time::Duration::from_millis(10)),
    )?;

    // Request name
    let reqname_serial = rpc_con.send_message(
        &mut standard_messages::request_name("io.killing.spark.dbustest".into(), 0),
        Timeout::Duration(std::time::Duration::from_millis(10)),
    )?;
    let _msg = rpc_con.wait_response(
        reqname_serial,
        Timeout::Duration(std::time::Duration::from_millis(10)),
    )?;

    let sig_serial = rpc_con.send_message(
        &mut standard_messages::add_match("type='signal'".into()),
        Timeout::Duration(std::time::Duration::from_millis(10)),
    )?;
    let _msg = rpc_con.wait_response(
        sig_serial,
        Timeout::Duration(std::time::Duration::from_millis(10)),
    )?;

    std::process::Command::new("dbus-send")
        .args(&[
            "--dest=io.killing.spark.dbustest",
            "/",
            "io.killing.spark.dbustest.Member",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    std::process::Command::new("dbus-send")
        .args(&[
            "--dest=io.killing.spark.dbustest",
            "/",
            "io.killing.spark.dbustest.Member",
            "string:ABCD",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    std::process::Command::new("dbus-send")
        .args(&[
            "--dest=io.killing.spark.dbustest",
            "/",
            "io.killing.spark.dbustest.Member",
            "array:string:ABCD,EFGH",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    std::process::Command::new("dbus-send")
        .args(&[
            "--dest=io.killing.spark.dbustest",
            "/",
            "io.killing.spark.dbustest.Member",
            "dict:uint32:string:100,ABCD,20,EFGH",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    std::process::Command::new("dbus-send")
        .args(&[
            "--dest=io.killing.spark.dbustest",
            "/",
            "io.killing.spark.dbustest.Member",
            "byte:10",
            "uint16:20",
            "uint64:30",
            "byte:40",
            "array:string:A,AB,ABC,ABCD,ABCDE,ABCDEF,ABCDEFG,ABCDEFGH",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    std::process::Command::new("dbus-send")
        .args(&[
            "--dest=io.killing.spark.dbustest",
            "/",
            "io.killing.spark.dbustest.Member",
            "array:uint64:10",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let msg = rpc_con
        .wait_signal(Timeout::Duration(std::time::Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        msg.dynheader.interface,
        Some("io.killing.spark.dbustest".to_owned())
    );
    assert_eq!(msg.dynheader.member, Some("Member".to_owned()));
    let msg = msg.unmarshall_all()?;
    assert_eq!(msg.params.len(), 0);

    let msg = rpc_con
        .wait_signal(Timeout::Duration(std::time::Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        msg.dynheader.interface,
        Some("io.killing.spark.dbustest".to_owned())
    );
    assert_eq!(msg.dynheader.member, Some("Member".to_owned()));
    let msg = msg.unmarshall_all()?;
    assert_eq!(msg.params.len(), 1);
    assert_eq!(msg.params[0].as_str().unwrap(), "ABCD");

    let msg = rpc_con
        .wait_signal(Timeout::Duration(std::time::Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        msg.dynheader.interface,
        Some("io.killing.spark.dbustest".to_owned())
    );
    assert_eq!(msg.dynheader.member, Some("Member".to_owned()));
    let strs: Vec<String> = msg.body.parser().get().unwrap();
    assert_eq!(strs[0], "ABCD");
    assert_eq!(strs[1], "EFGH");

    let msg = rpc_con
        .wait_signal(Timeout::Duration(std::time::Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        msg.dynheader.interface,
        Some("io.killing.spark.dbustest".to_owned())
    );
    assert_eq!(msg.dynheader.member, Some("Member".to_owned()));
    let strs: std::collections::HashMap<u32, String> = msg.body.parser().get().unwrap();
    assert_eq!(strs[&100], "ABCD");
    assert_eq!(strs[&20], "EFGH");

    let msg = rpc_con
        .wait_signal(Timeout::Duration(std::time::Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        msg.dynheader.interface,
        Some("io.killing.spark.dbustest".to_owned())
    );
    assert_eq!(msg.dynheader.member, Some("Member".to_owned()));
    let params: (u8, u16, u64, u8, Vec<&str>) = msg.body.parser().get5().unwrap();
    assert_eq!(params.0, 10);
    assert_eq!(params.1, 20);
    assert_eq!(params.2, 30);
    assert_eq!(params.3, 40);
    assert_eq!(
        params.4,
        ["A", "AB", "ABC", "ABCD", "ABCDE", "ABCDEF", "ABCDEFG", "ABCDEFGH"]
    );

    let msg = rpc_con
        .wait_signal(Timeout::Duration(std::time::Duration::from_millis(10)))
        .unwrap();
    assert_eq!(
        msg.dynheader.interface,
        Some("io.killing.spark.dbustest".to_owned())
    );
    assert_eq!(msg.dynheader.member, Some("Member".to_owned()));
    let ints: Vec<u64> = msg.body.parser().get().unwrap();
    assert_eq!(ints[0], 10);

    Ok(())
}
