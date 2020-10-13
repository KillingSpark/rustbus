use rustbus::{
    client_conn::Timeout, params::message::Message, standard_messages, MessageType, RpcConn,
};

pub enum Commands {
    Echo,
    Reverse(String),
}

impl<'a, 'e> Commands {
    fn execute(&self, call: &Message<'a, 'e>) -> rustbus::message_builder::MarshalledMessage {
        match self {
            Commands::Echo => {
                let mut reply = call.make_response();
                for p in &call.params {
                    reply.body.push_old_param(p).unwrap();
                }
                reply
            }
            Commands::Reverse(val) => {
                let mut reply = call.make_response();
                let reverse = val.chars().rev().collect::<String>();
                reply.body.push_param(reverse).unwrap();
                reply
            }
        }
    }
}

fn main() -> Result<(), rustbus::client_conn::Error> {
    // sends the obligatory hello message
    let mut rpc_con = RpcConn::session_conn(Timeout::Infinite)?;

    let namereq_serial = rpc_con.send_message(
        &mut standard_messages::request_name("io.killing.spark".into(), 0),
        Timeout::Infinite,
    )?;
    let resp = rpc_con.wait_response(namereq_serial, Timeout::Infinite)?;
    println!("Name request response: {:?}", resp);

    rpc_con.set_filter(Box::new(|msg| match msg.typ {
        MessageType::Call => {
            if rustbus::peer::filter_peer(&msg.dynheader) {
                true
            } else {
                let right_interface_object =
                    msg.dynheader.object.eq(&Some("/io/killing/spark".into()))
                        && msg.dynheader.interface.eq(&Some("io.killing.spark".into()));

                let right_member = if let Some(member) = &msg.dynheader.member {
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
        }
        MessageType::Invalid => false,
        MessageType::Error => true,
        MessageType::Reply => true,
        MessageType::Signal => false,
    }));

    loop {
        println!("Wait for call");
        let call = rpc_con.wait_call(Timeout::Infinite)?;
        let call = call.unmarshall_all()?;
        if rustbus::peer::handle_peer_message(&call, rpc_con.conn_mut(), Timeout::Infinite).unwrap()
        {
            continue;
        }
        println!("Got call: {:?}", call);
        if let Some(member) = &call.dynheader.member {
            let cmd = match member.as_str() {
                "Echo" => Commands::Echo,
                "Reverse" => {
                    if call.params.len() != 1 {
                        rpc_con.send_message(
                            &mut standard_messages::invalid_args(&call.dynheader, Some("String")),
                            Timeout::Infinite,
                        )?;
                        continue;
                    }
                    if let Some(val) = call.params[0].as_str() {
                        Commands::Reverse(val.to_owned())
                    } else {
                        rpc_con.send_message(
                            &mut standard_messages::invalid_args(&call.dynheader, Some("String")),
                            Timeout::Infinite,
                        )?;
                        continue;
                    }
                }
                _ => {
                    // This shouldn't happen with the filters defined above
                    // If a call is filtered out, an error like this one is automatically send to the source, so this is technically unecessary
                    // but we like robust software!
                    rpc_con.send_message(
                        &mut standard_messages::unknown_method(&call.dynheader),
                        Timeout::Infinite,
                    )?;
                    continue;
                }
            };
            let mut reply = cmd.execute(&call);
            //println!("Reply: {:?}", reply);
            rpc_con.send_message(&mut reply, Timeout::Infinite)?;
            println!("Reply sent");
        }
    }
}
