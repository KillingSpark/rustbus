use rustbus::message;
use rustbus::standard_messages;

pub enum Commands {
    Echo,
    Reverse(String),
}

impl Commands {
    fn execute(&self, call: &message::Message) -> message::Message {
        match self {
            Commands::Echo => {
                let mut reply = call.make_response();
                reply.push_params(call.params.clone());
                reply
            }
            Commands::Reverse(val) => {
                let mut reply = call.make_response();
                let reverse = val.chars().rev().collect::<String>();
                reply.push_params(vec![reverse.into()]);
                reply
            }
        }
    }
}

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = rustbus::client_conn::get_session_bus_path()?;
    let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true)?;
    let mut rpc_con = rustbus::client_conn::RpcConn::new(con);

    rpc_con.send_message(standard_messages::hello())?;

    let namereq_serial = rpc_con
        .send_message(standard_messages::request_name(
            "io.killing.spark".into(),
            0,
        ))?
        .serial
        .unwrap();
    let resp = rpc_con.wait_response(namereq_serial)?;
    println!("Name request response: {:?}", resp);

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        message::MessageType::Call => {
            let right_interface_object = msg.object.eq(&Some("/io/killing/spark".into()))
                && msg.interface.eq(&Some("io.killing.spark".into()));

            let right_member = if let Some(member) = &msg.member {
                member.eq("Echo") || member.eq("Reverse")
            } else {
                false
            };
            let keep = right_interface_object && right_member;
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
        let call = rpc_con.wait_call()?;
        println!("Got call: {:?}", call);
        if let Some(member) = &call.member {
            let cmd = match member.as_str() {
                "Echo" => Commands::Echo,
                "Reverse" => {
                    if call.params.len() != 1 {
                        rpc_con
                            .send_message(standard_messages::invalid_args(&call, Some("String")))?;
                        continue;
                    }
                    if let message::Param::Base(message::Base::String(val)) = &call.params[0] {
                        Commands::Reverse(val.clone())
                    } else {
                        rpc_con
                            .send_message(standard_messages::invalid_args(&call, Some("String")))?;
                        continue;
                    }
                }
                _ => {
                    // This shouldn't happen with the filters defined above
                    rpc_con.send_message(standard_messages::unknown_method(&call))?;
                    continue;
                }
            };
            let reply = cmd.execute(&call);
            println!("Reply: {:?}", reply);
            rpc_con.send_message(reply)?;
            println!("Reply sent");
        }
    }
}
