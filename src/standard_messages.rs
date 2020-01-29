use crate::message;
use crate::message_builder::MessageBuilder;

pub fn hello() -> message::Message {
    MessageBuilder::new()
        .call("Hello".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build()
}

pub fn ping(dest: String) -> message::Message {
    MessageBuilder::new()
        .call("Ping".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus.Peer".into())
        .at(dest)
        .build()
}

pub fn ping_bus() -> message::Message {
    MessageBuilder::new()
        .call("Ping".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus.Peer".into())
        .build()
}

pub fn list_names() -> message::Message {
    MessageBuilder::new()
        .call("ListNames".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build()
}

pub const DBUS_NAME_FLAG_ALLOW_REPLACEMENT: u32 = 1 << 0;
pub const DBUS_NAME_FLAG_REPLACE_EXISTING: u32 = 1 << 1;
pub const DBUS_NAME_FLAG_DO_NOT_QUEUE: u32 = 1 << 2;

pub const DBUS_REQUEST_NAME_REPLY_PRIMARY_OWNER: u32 = 1;
pub const DBUS_REQUEST_NAME_REPLY_IN_QUEUE: u32 = 2;
pub const DBUS_REQUEST_NAME_REPLY_EXISTS: u32 = 3;
pub const DBUS_REQUEST_NAME_REPLY_ALREADY_OWNER: u32 = 4;

pub fn request_name(name: String, flags: u32) -> message::Message {
    MessageBuilder::new()
        .call("RequestName".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .with_params(vec![
            message::Param::Base(message::Base::String(name)),
            message::Param::Base(message::Base::Uint32(flags)),
        ])
        .at("org.freedesktop.DBus".into())
        .build()
}

pub fn add_match(match_rule: String) -> message::Message {
    MessageBuilder::new()
        .call("AddMatch".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .with_params(vec![message::Param::Base(message::Base::String(
            match_rule,
        ))])
        .at("org.freedesktop.DBus".into())
        .build()
}
