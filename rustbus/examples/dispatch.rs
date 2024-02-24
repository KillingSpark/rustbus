use rustbus::connection::dispatch_conn::DispatchConn;
use rustbus::connection::dispatch_conn::HandleEnvironment;
use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::connection::ll_conn::DuplexConn;
use rustbus::message_builder::MarshalledMessage;

// just to make the function definitions a bit shorter
type MyHandleEnv<'a, 'b> = HandleEnvironment<&'b mut Counter, ()>;

struct Counter {
    count: u64,
}
fn default_handler(
    c: &mut &mut Counter,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
) -> HandleResult<()> {
    c.count += 1;
    println!(
        "Woohoo the default handler got called for \"{:?}\" (the {}'ths time)",
        msg.dynheader.object, c.count
    );
    Ok(None)
}
fn name_handler(
    c: &mut &mut Counter,
    matches: Matches,
    _msg: &MarshalledMessage,
    env: &mut MyHandleEnv,
) -> HandleResult<()> {
    c.count += 1;
    println!(
        "Woohoo a name got called (the {}'ths time): {}",
        c.count,
        matches.matches.get(":name").unwrap()
    );

    let mut name_counter = Counter { count: 0 };
    let name = matches.matches.get(":name").unwrap().to_owned();
    let ch = Box::new(
        move |c: &mut &mut Counter,
              _matches: Matches,
              _msg: &MarshalledMessage,
              _env: &mut MyHandleEnv| {
            name_counter.count += 1;
            c.count += 1;

            println!(
                "Woohoo the closure for {} got called (the {}'ths time)",
                name, name_counter.count
            );
            Ok(None)
        },
    );

    let new_path = format!("/{}", matches.matches.get(":name").unwrap());
    println!("Add new path: \"{}\"", new_path);

    env.new_dispatches.insert(&new_path, ch);

    Ok(None)
}

fn main() {
    let mut con =
        DuplexConn::connect_to_bus(rustbus::connection::get_session_bus_path().unwrap(), false)
            .unwrap();
    con.send_hello(rustbus::connection::Timeout::Infinite)
        .unwrap();

    if std::env::args().any(|arg| "server".eq(&arg)) {
        con.send
            .send_message(&mut rustbus::standard_messages::request_name(
                "killing.spark.io",
                rustbus::standard_messages::DBUS_NAME_FLAG_REPLACE_EXISTING,
            ))
            .unwrap()
            .write_all()
            .unwrap();

        let mut counter = Counter { count: 0 };
        let dh = Box::new(default_handler);
        let nh = Box::new(name_handler);
        let ch = Box::new(
            |c: &mut &mut Counter,
             _matches: Matches,
             _msg: &MarshalledMessage,
             _env: &mut MyHandleEnv| {
                c.count += 1;
                println!("Woohoo the closure got called (the {}'ths time)", c.count,);
                Ok(None)
            },
        );
        let mut dpcon = DispatchConn::new(con, &mut counter, dh);
        dpcon.add_handler("/A/B/:name", nh);
        dpcon.add_handler("/A/C/D", ch);

        dpcon.run().unwrap();
    } else {
        println!("Sending stuff!");

        // default handler
        let mut msg1 = rustbus::message_builder::MessageBuilder::new()
            .call("ABCD")
            .at("killing.spark.io")
            .on("/ABCD")
            .build();
        con.send
            .send_message(&mut msg1)
            .unwrap()
            .write_all()
            .unwrap();

        // pick up the name
        let mut msg2 = rustbus::message_builder::MessageBuilder::new()
            .call("ABCD")
            .at("killing.spark.io")
            .on("/A/B/moritz")
            .build();
        con.send
            .send_message(&mut msg2)
            .unwrap()
            .write_all()
            .unwrap();

        // call new handler for that name
        let mut msg3 = rustbus::message_builder::MessageBuilder::new()
            .call("ABCD")
            .at("killing.spark.io")
            .on("/moritz")
            .build();
        con.send
            .send_message(&mut msg3)
            .unwrap()
            .write_all()
            .unwrap();
        con.send
            .send_message(&mut msg3)
            .unwrap()
            .write_all()
            .unwrap();
        con.send
            .send_message(&mut msg3)
            .unwrap()
            .write_all()
            .unwrap();
    }
}
