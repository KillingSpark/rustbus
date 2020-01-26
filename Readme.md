# Rustbus
Rustbus is an implementation of the dbus client protocol like libdbus. Currently WIP, many parts are unimplemented but pretty much the whole 
(un)marshaling code is there.

This was created by only reading the spec at https://dbus.freedesktop.org/doc/dbus-specification.html. While I made some false assumptions when implementing the 
spec that was mostly my fault. The document seems to be enough to write a working implementation without looking at others code.

## What does this provide?
Currently only (un)marshaling of dbus messages. There is a small implementation around reading/writing messages from unix sockets. There is also 
a simple auth package that only supports the external authentication (which is enough if you only use unix sockets, like your local dbus session/system bus).