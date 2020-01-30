# Rustbus
[![Actions Status](https://github.com/KillingSpark/rustbus/workflows/CI/badge.svg)](https://github.com/KillingSpark/rustbus/actions?query=workflow%3A"CI")

Rustbus implements the [dbus specification](https://dbus.freedesktop.org/doc/dbus-specification.html) for local unix sockets. It is not a bus implementation but a library
that enables clients to communicate over the dbus daemon.

This was created by only reading the spec at https://dbus.freedesktop.org/doc/dbus-specification.html. While I made some false assumptions when implementing the 
spec that was mostly my fault. The document seems to be enough to write a working implementation without looking at others code. The example in src/bin/con.rs is able
to listen to signals on the message bus without crashing/getting  disconnected by the bus. The signals parsed by this are the same as the ones dbus-monitor sees / unmarshals.

## What does this provide?
This libary provides the means to send and receive messages over a dbus bus. This means: signals, calls, and (error)replys. It also provides some standard messages
for convenience. There is also a MessageBuilder to help you conveniently build your own messages.

Dbus does technically work over any transport but this currently only supports unix streaming sockets. Support for other transports should be rather simple. The problem is
that to use transports like tcp require other authentication methods which is the complicated part.

Transmitting filedescriptors works. The limit is currently set to 10 per message but since dbus-daemon limits this even more this should be fine.

## State of this project
Generally working but there are probably bugs lingering. Need to setup fuzzing and unit tests.


# How to use it
There are some examples in the `examples/` directory but the gist is:
```
// Connect to the session bus
let session_path = rustbus::client_conn::get_session_bus_path()?;
let con = rustbus::client_conn::Conn::connect_to_bus(session_path, true)?;

// Wrap the con in an RpcConnection which provides many convenient functions
let mut rpc_con = rustbus::client_conn::RpcConn::new(con);

// send the obligatory hello message
rpc_con.send_message(standard_messages::hello())?;

// Request a bus name if you want to
rpc_con.send_message(standard_messages::request_name(
    "io.killing.spark".into(),
    0,
))?;

// send a signal to all bus members
let sig = MessageBuilder::new()
.signal(
    "io.killing.spark".into(),
    "TestSignal".into(),
    "/io/killing/spark".into(),
)
.with_params(vec![
    Container::Array(vec!["ABCDE".to_owned().into()]).into(),
    Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
    Container::Array(vec![
        Container::Struct(vec![162254319i32.into(), "AABB".to_owned().into()]).into(),
        Container::Struct(vec![305419896i32.into(), "CCDD".to_owned().into()]).into(),
    ])
    .into(),
    Container::Dict(dict).into(),
])
.build();
con.send_message(sig)?;
```