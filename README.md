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

## What's where?
* `rustbus` is the core crate containing bus-connection and (un)-marshalling code. If you want to write an application you only need this.
* `rustbus_derive` contains the procmacros to derive the (Un-)Marshal traits for structs. The macros are re-exported by rustbus so you dont need to worry about that.
* `rustbus_derive_test` is only there to verify that the derives do the right things. procmacro crates apparently can't contain tests themselves.
* `example_keywallet` is there as
    * a more complex example showcasing rustbus
    * a testing ground for new ideas to validate how it would impact actual development


## Quickstart
```rust
use rustbus::{connection::{Timeout, ll_conn::force_finish_on_error}, standard_messages, MessageBuilder, MessageType, RpcConn};

fn main() {
    // Connect to the session bus. This also takes care of the mandatory hello messages
    let mut rpc_con = RpcConn::session_conn(Timeout::Infinite).unwrap();

    // Create a new call to a method on a remote object
    let mut call = MessageBuilder::new()
        .call("evaluateScript")
        .with_interface("org.kde.PlasmaShell")
        .on("/PlasmaShell")
        .at("org.kde.plasmashell")
        .build();

    // Send the message and remeber the id, so we can retrieve the answer
    let id = rpc_con
        .send_message(&mut call)
        .expect("Wanna send message :(")
        .write_all()
        .map_err(force_finish_on_error)
        .expect("Wanna send message :(");

    // Retrieve the answer to this message
    let message = rpc_con
        .wait_response(id, Timeout::Infinite)
        .expect("Get failed");
}
```

 ## Connection Types
 * Low level connection is the basis for building more abstract wrappers. You probably don't want to use it outside of special cases.
 * RpcConn is meant for clients calling methods on services on the bus (as shown in the quick start)
 * DispatchConn is meant for services that need to dispatch calls to many handlers.

 Since different usecases have different constraints you might need to write your own wrapper around the low level conn. This should not be too hard
 if you copy the existing ones and modify them to your needs. If you have an issue that would be helpful for others I would of course consider adding
 it to this libary.

 ### Low level connection example
 ```rust
use rustbus::{connection::Timeout, get_session_bus_path, DuplexConn, MessageBuilder};
fn main() -> Result<(), rustbus::connection::Error> {
    // To get a connection going you need to connect to a bus. You will likely use either the session or the system bus.
    let session_path = get_session_bus_path()?;
    let mut con: DuplexConn = DuplexConn::connect_to_bus(session_path, true)?;
    // Dont forget to send the **mandatory** hello message. send_hello wraps the call and parses the response for convenience.
    let _unique_name: String = con.send_hello(Timeout::Infinite)?;

    // Next you will probably want to create a new message to send out to the world
    let mut sig = MessageBuilder::new()
        .signal("io.killing.spark", "TestSignal", r#"/io/killing/spark"#)
        .build();

    // To put parameters into that message you use the sig.body.push_param functions. These accept anything that can be marshalled into a dbus parameter
    // You can derive or manually implement that trait for your own types if you need that.
    sig.body.push_param("My cool new Signal!").unwrap();

    // Now send you signal to all that want to hear it!
    con.send.send_message(&sig)?.write_all().unwrap();

    // To receive messages sent to you you can call the various functions on the RecvConn. The simplest is this:
    let message = con.recv.get_next_message(Timeout::Infinite)?;

    // Now you can inspect the message.dynheader for all the metadata on the message
    println!("The messages dynamic header: {:?}", message.dynheader);

    // After inspecting that dynheader you should know which content the message should contain
    let cool_string = message.body.parser().get::<&str>().unwrap();
    println!("Received a cool string: {}", cool_string);
    Ok(())
}
```

 ## Params and Marshal and Unmarshal
 This lib started out as an attempt to understand how dbus worked. Thus I modeled the types a closely as possible with enums, which is still in the params module.
 This is kept around for weird weird edge-cases where that might be necessary but they should not generally be used.

 Instead you should be using the Marshal and Unmarshal traits which are implemented for most common types you will need. The idea is to map rust types
 as closely as possible to dbus types. The trivial types like String and u64 etc are dealt with easily. For tuple-structs there are impls up to a
 certain size. After that you'd need to copy the impl from this lib and extend it accordingly. This might be dealt with in the future if variadic generics get
 added to rust.

 For structs there is a derive proc-macro that derives the necessary trait impls for you. Look into rustbus_derive if this is of need for you.

 For enums there is also a proc-macro that derives the necessary trait impls for you. There are two legacy macros: `dbus_variant_sig!` and `dbus_variant_var!`.
 They do effectively the same, but the legacy macros add a `CatchAll` to our enum to help with unexpected types, where the proc-macros fails unmarshalling with an error.

 The doc for the traits gives more specifics on how to implement them for your own types if necessary.

 There is an exmaple for all of this in `examples/user_defined_types.rs`.
 And for the deriving for structs there is an example in `examples/deriving.rs`

 ## Filedescriptors
 Dbus can send filedescriptors around for you. Rustbus supports this. There is a special wrapper type in the wire module. This type tries to sensibly deal with
 the pitfalls of sending and receiving filedescriptors in a sensible way. If you see any issues with the API or have wishes for extensions to the API please
 open an issue.

 ## Byteorders
 Dbus supports both big and little endian and so does rustbus. You can specify how a message should be marshalled when you create the MessageBuilder. Messages
 can be received in any byteorder and will be transparently unmarshalled into the byteorder you CPU uses. Note that unmarshalling from/to the native byteorder will
 be faster. The default byteorder is the native byteorder of your compilation target.
