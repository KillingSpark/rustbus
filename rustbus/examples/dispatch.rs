use rustbus::connection::dispatch_conn::DispatchConn;
use rustbus::connection::dispatch_conn::HandleFn;
use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::connection::ll_conn::Conn;
use rustbus::message_builder::MarshalledMessage;

struct Counter {
    count: u64,
}
fn default_handler(
    c: &mut &mut Counter,
    _matches: Matches,
    _msg: &MarshalledMessage,
) -> HandleResult<()> {
    c.count += 1;
    println!(
        "Woohoo the default handler got called (the {}'ths time)",
        c.count
    );
    Ok(())
}
fn name_handler(
    c: &mut &mut Counter,
    matches: Matches,
    _msg: &MarshalledMessage,
) -> HandleResult<()> {
    c.count += 1;
    println!(
        "Woohoo a name got called (the {}'ths time): {}",
        c.count,
        matches.matches.get(":name").unwrap()
    );
    Ok(())
}

fn main() {
    let mut con =
        Conn::connect_to_bus(rustbus::connection::get_session_bus_path().unwrap(), false).unwrap();
    con.send_message(
        &mut rustbus::standard_messages::hello(),
        rustbus::connection::Timeout::Infinite,
    )
    .unwrap();

    if std::env::args().find(|arg| "server".eq(arg)).is_some() {
        con.send_message(
            &mut rustbus::standard_messages::request_name(
                "killing.spark.io".into(),
                rustbus::standard_messages::DBUS_NAME_FLAG_REPLACE_EXISTING,
            ),
            rustbus::connection::Timeout::Infinite,
        )
        .unwrap();

        let mut counter = Counter { count: 0 };
        let dh: &mut HandleFn<&mut Counter, ()> = &mut default_handler;
        let nh: &mut HandleFn<&mut Counter, ()> = &mut name_handler;
        let ch: &mut HandleFn<&mut Counter, ()> =
            &mut |c: &mut &mut Counter, _matches: Matches, _msg: &MarshalledMessage| {
                c.count += 1;
                println!("Woohoo the closure got called (the {}'ths time)", c.count,);
                Ok(())
            };
        let mut dpcon = DispatchConn::new(con, &mut counter, dh);
        dpcon.add_handler("/A/B/:name", nh);
        dpcon.add_handler("/A/C/D", ch);

        dpcon.run();
    } else {
        println!("Sending stuff!");
        let mut msg1 = rustbus::message_builder::MessageBuilder::new()
            .call("ABCD".into())
            .at("killing.spark.io".into())
            .on("/ABCD".into())
            .build();
        con.send_message(&mut msg1, rustbus::connection::Timeout::Infinite)
            .unwrap();

        let mut msg2 = rustbus::message_builder::MessageBuilder::new()
            .call("ABCD".into())
            .at("killing.spark.io".into())
            .on("/A/B/moritz".into())
            .build();
        con.send_message(&mut msg2, rustbus::connection::Timeout::Infinite)
            .unwrap();
        let mut msg2 = rustbus::message_builder::MessageBuilder::new()
            .call("ABCD".into())
            .at("killing.spark.io".into())
            .on("/A/C/D".into())
            .build();
        con.send_message(&mut msg2, rustbus::connection::Timeout::Infinite)
            .unwrap();
    }
}
