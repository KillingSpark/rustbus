use rustbus::{
    get_session_bus_path, params, standard_messages, Conn, Message, MessageType, RpcConn,
};

pub enum Commands {
    Echo,
    Reverse(String),
}

impl<'a, 'e> Commands {
    fn execute(&self, call: &Message<'a, 'e>) -> Message<'a, 'e> {
        match self {
            Commands::Echo => {
                let mut reply = call.make_response();
                reply.push_params(call.params.clone());
                reply
            }
            Commands::Reverse(val) => {
                let mut reply = call.make_response();
                let reverse = val.chars().rev().collect::<String>();
                reply.push_param(reverse.into());
                reply
            }
        }
    }
}

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let con = Conn::connect_to_bus(session_path, true)?;
    let mut rpc_con = RpcConn::new(con);

    rpc_con.send_message(&mut standard_messages::hello(), None)?;

    let namereq_serial = rpc_con.send_message(
        &mut standard_messages::request_name("io.killing.spark".into(), 0),
        None,
    )?;
    let resp = rpc_con.wait_response(namereq_serial, None)?;
    println!("Name request response: {:?}", resp);

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        MessageType::Call => {
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
        MessageType::Invalid => false,
        MessageType::Error => true,
        MessageType::Reply => true,
        MessageType::Signal => false,
    }));

    loop {
        println!("Wait for call");
        let call = rpc_con.wait_call(None)?;
        println!("Got call: {:?}", call);
        if let Some(member) = &call.member {
            let cmd = match member.as_str() {
                "Echo" => Commands::Echo,
                "Reverse" => {
                    if call.params.len() != 1 {
                        rpc_con.send_message(
                            &mut standard_messages::invalid_args(&call, Some("String")),
                            None,
                        )?;
                        continue;
                    }
                    if let params::Param::Base(params::Base::String(val)) = &call.params[0] {
                        Commands::Reverse(val.clone())
                    } else {
                        rpc_con.send_message(
                            &mut standard_messages::invalid_args(&call, Some("String")),
                            None,
                        )?;
                        continue;
                    }
                }
                _ => {
                    // This shouldn't happen with the filters defined above
                    rpc_con.send_message(&mut standard_messages::unknown_method(&call), None)?;
                    continue;
                }
            };
            let mut reply = cmd.execute(&call);
            println!("Reply: {:?}", reply);
            rpc_con.send_message(&mut reply, None)?;
            println!("Reply sent");
        }
    }
}
