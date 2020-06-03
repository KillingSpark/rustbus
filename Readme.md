# Rustbus
[![Actions Status](https://github.com/KillingSpark/rustbus/workflows/CI/badge.svg)](https://github.com/KillingSpark/rustbus/actions?query=workflow%3A"CI")

Rustbus implements the [dbus specification](https://dbus.freedesktop.org/doc/dbus-specification.html) for local unix sockets. It is not a bus implementation but a library
that enables clients to communicate over the dbus daemon.

This was created by only reading the spec at https://dbus.freedesktop.org/doc/dbus-specification.html. While I made some false assumptions when implementing the 
spec that was mostly my fault. The document seems to be enough to write a working implementation without looking at others code. 

## What does this provide?
This libary provides the means to send and receive messages over a dbus bus. This means: signals, calls, and (error)replys. It also provides some standard messages
for convenience. There is also a MessageBuilder to help you conveniently build your own messages.

Dbus does technically work over any transport but this currently only supports unix streaming sockets. Support for other transports should be rather simple, but 
they require the implementation of some other authentication mechanisms.

Transmitting filedescriptors works. The limit is currently set to 10 per message but since dbus-daemon limits this even more this should be fine.

## State of this project
There are some tests for correctness and the dbus-daemon seems to generally accept all messages sent by this lib. 

Interoperability with libdbus is not yet thorougly tested, but the dbus-monitor tool correctly displays the signals sent in the 'examples/sig.rs' 
example which uses pretty much all different types that can occur.

The unmarshalling has been fuzzed and doesn't panic on any input so far. If you wanto to help fuzzing, just use the command: `cargo +nightly fuzz run fuzz_unmarshal` 

The API is still very much in progress and breaking changes are to be expected.

# How to use it
There are some examples in the `examples/` directory but the gist is:
```rust
use rustbus::{get_session_bus_path, standard_messages, Conn, Container, params::DictMap, MessageBuilder, client_conn::Timeout};

fn main() -> Result<(), rustbus::client_conn::Error> {
    // Connect to the session bus
    let mut rpc_con = RpcConn::session_conn(Timeout::Infinite)?;

    // send the obligatory hello message
    rpc_con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;

    // Request a bus name if you want to
    rpc_con.send_message(&mut standard_messages::request_name(
        "io.killing.spark".into(),
        0,
    ), Timeout::Infinite)?;

    // create a signal with the MessageBuilder API
    let mut sig = MessageBuilder::new()
    .signal(
        "io.killing.spark".into(),
        "TestSignal".into(),
        "/io/killing/spark".into(),
    )
    .build();
    
    // add a parameter to the signal
    sig.body.push_param("Signal message!").unwrap();

    // send a signal to all bus members
    rpc_con.send_message(&mut sig, Timeout::Infinite)?;
    Ok(())
}
```
