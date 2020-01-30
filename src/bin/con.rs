extern crate rustbus;
use rustbus::standard_messages;

fn main() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
    let mut rpc_con = rustbus::client_conn::RpcConn::new(con);

    let hello_msg = standard_messages::hello();

    println!("Send message: {:?}", hello_msg);
    let hello_serial = rpc_con.send_message(hello_msg).unwrap().serial.unwrap();

    println!("\n");
    println!("\n");
    println!("\n");
    println!("Wait for hello response");
    let msg = rpc_con.wait_response(&hello_serial).unwrap();
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let reqname_serial = rpc_con
        .send_message(standard_messages::request_name(
            "io.killing.spark".into(),
            0,
        ))
        .unwrap()
        .serial
        .unwrap();

    println!("Wait for name request response");
    let msg = rpc_con.wait_response(&reqname_serial).unwrap();
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    if let rustbus::message::Param::Base(rustbus::message::Base::Uint32(ret)) = msg.params[0] {
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
        .send_message(standard_messages::list_names())
        .unwrap()
        .serial
        .unwrap();

    println!("Wait for list response");
    let msg = rpc_con.wait_response(&list_serial).unwrap();
    println!("Got response: {:?}", msg);
    println!("\n");
    println!("\n");
    println!("\n");

    let sig_listen_msg = standard_messages::add_match("type='signal'".into());

    println!("Send message: {:?}", sig_listen_msg);
    rpc_con.send_message(sig_listen_msg).unwrap();

    loop {
        println!("Wait for incoming signals");
        let msg = rpc_con.wait_signal().unwrap();
        println!("Got signal: {:?}", msg);
        loop {
            let msg = rpc_con.try_get_signal();
            if let Some(msg) = msg {
                println!("Got signal: {:?}", msg);
            }else{
                break;
            }
        }
        println!("\n");
        println!("\n");
        println!("\n");
    }
}
