//! Some standard messages that are often needed

use crate::message;
use crate::message_builder::MessageBuilder;
use crate::message_builder::OutMessage;

pub fn hello<'a, 'e>() -> OutMessage {
    MessageBuilder::new()
        .call("Hello".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build()
}

pub fn ping<'a, 'e>(dest: String) -> OutMessage {
    MessageBuilder::new()
        .call("Ping".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus.Peer".into())
        .at(dest)
        .build()
}

pub fn ping_bus<'a, 'e>() -> OutMessage {
    MessageBuilder::new()
        .call("Ping".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus.Peer".into())
        .build()
}

pub fn list_names<'a, 'e>() -> OutMessage {
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
pub fn request_name<'a, 'e>(name: String, flags: u32) -> OutMessage {
    let mut msg = MessageBuilder::new()
        .call("RequestName".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build();

    msg.body.push_param(name.as_str()).unwrap();
    msg.body.push_param(flags).unwrap();
    msg
}

/// Add a match rule to receive signals. e.g. match_rule = "type='signal'" to get all signals
pub fn add_match<'a, 'e>(match_rule: String) -> OutMessage {
    let mut msg = MessageBuilder::new()
        .call("AddMatch".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build();

    msg.body.push_param(match_rule).unwrap();
    msg
}

/// Error message to tell the caller that this method is not known by your server
pub fn unknown_method<'a, 'e>(call: &message::Message<'a, 'e>) -> OutMessage {
    let text = format!(
        "No calls to {}.{} are accepted for object {}",
        call.interface.clone().unwrap_or_else(|| "".to_owned()),
        call.member.clone().unwrap_or_else(|| "".to_owned()),
        call.object.clone().unwrap_or_else(|| "".to_owned()),
    );
    call.make_error_response(
        "org.freedesktop.DBus.Error.UnknownMethod".to_owned(),
        Some(text),
    )
}

/// Error message to tell the caller that this method uses a different interface than what the caller provided as parameters
pub fn invalid_args<'a, 'e>(call: &message::Message<'a, 'e>, sig: Option<&str>) -> OutMessage {
    let text = format!(
        "Invalid arguments for calls to {}.{} on object {} {}",
        call.interface.clone().unwrap_or_else(|| "".to_owned()),
        call.member.clone().unwrap_or_else(|| "".to_owned()),
        call.object.clone().unwrap_or_else(|| "".to_owned()),
        if let Some(sig) = sig {
            format!("expected signature: {}", sig)
        } else {
            "".to_owned()
        }
    );

    call.make_error_response(
        "org.freedesktop.DBus.Error.InvalidArgs".to_owned(),
        Some(text),
    )
}
