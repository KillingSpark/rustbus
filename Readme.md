# Rustbus
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
