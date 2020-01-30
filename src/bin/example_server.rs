extern crate rustbus;
use rustbus::message;
use rustbus::standard_messages;

fn main() {
    let session_path = rustbus::client_conn::get_session_bus_path().unwrap();
    let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true).unwrap();
    let mut rpc_con = rustbus::client_conn::RpcConn::new(con);

    rpc_con
        .send_message(standard_messages::hello())
        .unwrap();

    let namereq_serial = rpc_con
        .send_message(standard_messages::request_name("io.killing.spark".into(), 0))
        .unwrap()
        .serial
        .unwrap();
    
    let resp = rpc_con.wait_response(&namereq_serial).unwrap();
    println!("Name request response: {:?}", resp);
    

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        message::MessageType::Call => {
            let keep = msg.object.eq(&Some("/io/killing/spark".into()))
                && msg.interface.eq(&Some("io.killing.spark".into()));
            if !keep {
                println!("Discard: {:?}", msg);
            }
            keep
        }
        message::MessageType::Invalid => false,
        message::MessageType::Error => true,
        message::MessageType::Reply => true,
        message::MessageType::Signal => false,
    }));

    loop {
        println!("Wait for call");
        let call = rpc_con.wait_call().unwrap();
        println!("Got call: {:?}", call);
        if let Some(member) = &call.member {
            match member.as_str() {
                "Echo" => {
                    let mut reply = call.make_response();
                    reply.push_params(call.params.clone());
                    println!("Echo: {:?}", reply);
                    rpc_con.send_message(reply).unwrap();
                    println!("Echo sent");
                }
                _ => {
                    rpc_con.send_message(standard_messages::unknown_method(&call)).unwrap();
                }
            }
        }
    }
}
