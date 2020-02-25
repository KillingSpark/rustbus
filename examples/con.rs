use rustbus::{get_session_bus_path, standard_messages, Conn, MessageType, RpcConn};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let con = Conn::connect_to_bus(session_path, true)?;
    let mut rpc_con = RpcConn::new(con);

    let hello_msg = standard_messages::hello();

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        MessageType::Call => false,
        MessageType::Invalid => false,
        MessageType::Error => true,
        MessageType::Reply => true,
        MessageType::Signal => msg.interface.eq(&Some("io.killing.spark".to_owned())),
    }));

    println!("Send message: {:?}", hello_msg);
    let hello_serial = rpc_con.send_message(hello_msg, None)?.serial.unwrap();

    println!("\n");
    println!("\n");
    println!("\n");
    println!("Wait for hello response");
    let msg = rpc_con.wait_response(hello_serial, None)?;
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let reqname_serial = rpc_con
        .send_message(
            standard_messages::request_name("io.killing.spark".into(), 0),
            None,
        )?
        .serial
        .unwrap();

    println!("Wait for name request response");
    let msg = rpc_con.wait_response(reqname_serial, None)?;
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    if let rustbus::params::Param::Base(rustbus::params::Base::Uint32(ret)) = msg.params[0] {
        match ret {
            standard_messages::DBUS_REQUEST_NAME_REPLY_PRIMARY_OWNER => {
                println!("Got name");
            }
            _ => panic!("Got other return: {}", ret),
        }
    } else {
        panic!("Wrong args: {:?}", msg.params);
    }

    let list_serial = rpc_con
        .send_message(standard_messages::list_names(), None)?
        .serial
        .unwrap();

    println!("Wait for list response");
    let msg = rpc_con.wait_response(list_serial, None)?;
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let sig_listen_msg = standard_messages::add_match("type='signal'".into());

    println!("Send message: {:?}", sig_listen_msg);
    rpc_con.send_message(sig_listen_msg, None)?;

    loop {
        println!("Wait for incoming signals");
        let msg = rpc_con.wait_signal(Some(std::time::Duration::from_secs(5)))?;
        println!("Got signal: {:?}", msg);
        loop {
            let msg = rpc_con.try_get_signal();
            if let Some(msg) = msg {
                println!("Got signal: {:?}", msg);
            } else {
                break;
            }
        }
        println!("\n");
        println!("\n");
        println!("\n");
    }
}
