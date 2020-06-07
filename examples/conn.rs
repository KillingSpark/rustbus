use rustbus::{
    client_conn::Timeout, get_session_bus_path, standard_messages, Conn, MessageType, RpcConn,
};

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let con = Conn::connect_to_bus(session_path, true)?;
    let mut rpc_con = RpcConn::new(con);

    let mut hello_msg = standard_messages::hello();

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        MessageType::Call => false,
        MessageType::Invalid => false,
        MessageType::Error => true,
        MessageType::Reply => true,
        MessageType::Signal => msg
            .dynheader
            .interface
            .eq(&Some("io.killing.spark".to_owned())),
    }));

    //println!("Send message: {:?}", hello_msg);
    let hello_serial = rpc_con.send_message(&mut hello_msg, Timeout::Infinite)?;

    println!("\n");
    println!("\n");
    println!("\n");
    println!("Wait for hello response");
    let msg = rpc_con.wait_response(hello_serial, Timeout::Infinite)?;
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let reqname_serial = rpc_con.send_message(
        &mut standard_messages::request_name("io.killing.spark".into(), 0),
        Timeout::Infinite,
    )?;

    println!("Wait for name request response");
    let msg = rpc_con.wait_response(reqname_serial, Timeout::Infinite)?;
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let msg = msg.unmarshall_all()?;

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

    let list_serial =
        rpc_con.send_message(&mut standard_messages::list_names(), Timeout::Infinite)?;

    println!("Wait for list response");
    let msg = rpc_con.wait_response(list_serial, Timeout::Infinite)?;
    let msg = msg.unmarshall_all()?;
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let mut sig_listen_msg = standard_messages::add_match("type='signal'".into());

    //println!("Send message: {:?}", sig_listen_msg);
    rpc_con.send_message(&mut sig_listen_msg, Timeout::Infinite)?;

    loop {
        println!("Do important work while signals might arrive");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!("Collect all signals");
        rpc_con.refill_all()?;

        println!("Refill ended, now pull all signals out of the queue");
        loop {
            let msg = rpc_con.try_get_signal();
            if let Some(msg) = msg {
                let msg = msg.unmarshall_all()?;
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
