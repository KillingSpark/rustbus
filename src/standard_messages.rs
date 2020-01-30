//! Some standard messages that are often needed

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

pub const DBUS_NAME_FLAG_ALLOW_REPLACEMENT: u32 = 1;
pub const DBUS_NAME_FLAG_REPLACE_EXISTING: u32 = 1 << 1;
pub const DBUS_NAME_FLAG_DO_NOT_QUEUE: u32 = 1 << 2;

pub const DBUS_REQUEST_NAME_REPLY_PRIMARY_OWNER: u32 = 1;
pub const DBUS_REQUEST_NAME_REPLY_IN_QUEUE: u32 = 2;
pub const DBUS_REQUEST_NAME_REPLY_EXISTS: u32 = 3;
pub const DBUS_REQUEST_NAME_REPLY_ALREADY_OWNER: u32 = 4;

/// Request a name on the bus
pub fn request_name(name: String, flags: u32) -> message::Message {
    MessageBuilder::new()
        .call("RequestName".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .with_params(vec![name.into(), flags.into()])
        .at("org.freedesktop.DBus".into())
        .build()
}

/// Add a match rule to receive signals. e.g. match_rule = "type='signal'" to get all signals
pub fn add_match(match_rule: String) -> message::Message {
    MessageBuilder::new()
        .call("AddMatch".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .with_params(vec![match_rule.into()])
        .at("org.freedesktop.DBus".into())
        .build()
}

/// Error message to tell the caller that this method is not known by your server
pub fn unknown_method(call: &message::Message) -> message::Message {
    let mut reply = call.make_error_response("org.freedesktop.DBus.Error.UnknownMethod".to_owned());
    reply.push_params(vec![format!(
        "No calls to {}.{} are accepted for object {}",
        call.interface.clone().unwrap_or_else(|| "".to_owned()),
        call.member.clone().unwrap_or_else(|| "".to_owned()),
        call.object.clone().unwrap_or_else(|| "".to_owned()),
    )
    .into()]);
    reply
}

/// Error message to tell the caller that this method uses a different interface than what the caller provided as parameters
pub fn invalid_args(call: &message::Message, sig: Option<&str>) -> message::Message {
    let mut reply = call.make_error_response("org.freedesktop.DBus.Error.InvalidArgs".to_owned());
    reply.push_params(vec![format!(
        "Invalid arguments for calls to {}.{} on object {} {}",
        call.interface.clone().unwrap_or_else(|| "".to_owned()),
        call.member.clone().unwrap_or_else(|| "".to_owned()),
        call.object.clone().unwrap_or_else(|| "".to_owned()),
        if let Some(sig) = sig {
            format!("expected signature: {}", sig)
        } else {
            "".to_owned()
        }
    )
    .into()]);
    reply
}
