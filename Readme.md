# Rustbus
Rustbus is an implementation of the dbus client protocol like libdbus. Currently WIP, many parts are unimplemented but pretty much the whole 
(un)marshaling code is there.

This was created by only reading the spec at https://dbus.freedesktop.org/doc/dbus-specification.html. While I made some false assumptions when implementing the 
spec that was mostly my fault. The document seems to be enough to write a working implementation without looking at others code. The example in src/bin/con.rs is able
to listen to signals on the message bus without crashing/getting  disconnected by the bus. The signals parsed by this are the same as the ones dbus-monitor sees / unmarshals.

## What does this provide?
Currently only (un)marshaling of dbus messages. There is a small implementation around reading/writing messages from unix sockets. There is also 
a simple auth package that only supports the external authentication (which is enough if you only use unix sockets, like your local dbus session/system bus).

Dbus does technically work over any transport but this currently only supports unix streaming sockets. Support for other transports should be rather simple.

Receiving filedescriptors is implemented but untested.

## State of this project
Working for simple cases but there are probably bugs lingering. Need to setup fuzzing and unit tests.
