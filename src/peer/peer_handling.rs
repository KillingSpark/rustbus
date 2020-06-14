use crate::message_builder::DynamicHeader;
use crate::params::message::Message;
use crate::*;

static MACHINE_ID_FILE_PATH: &str = "/tmp/dbus_machine_uuid";

/// Can be used in the RpcConn filters to allow for peer messages
pub fn filter_peer(msg: &DynamicHeader) -> bool {
    if let Some(interface) = &msg.interface {
        if interface.eq("org.freedesktop.DBus.Peer") {
            if let Some(member) = &msg.member {
                match member.as_str() {
                    "Ping" => true,
                    "GetMachineId" => true,

                    // anything else is not in this interface and thus not handled here
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

fn create_and_store_machine_uuid() -> Result<(), std::io::Error> {
    let now = std::time::SystemTime::now();
    let secs = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
    let mut rand = [0u8; 12];

    let mut rand_file = std::fs::File::open("/dev/urandom").unwrap();
    use std::io::Read;
    rand_file.read_exact(&mut rand[..]).unwrap();

    let rand1 = rand[0] as u64
        | ((rand[1] as u64) << 8)
        | ((rand[2] as u64) << 16)
        | ((rand[3] as u64) << 24)
        | ((rand[4] as u64) << 32)
        | ((rand[5] as u64) << 40)
        | ((rand[6] as u64) << 48)
        | ((rand[7] as u64) << 56);
    let rand2 = rand[8] as u32
        | ((rand[9] as u32) << 8)
        | ((rand[1] as u32) << 16)
        | ((rand[11] as u32) << 24);

    let uuid = format!("{:08X}{:04X}{:04X}", rand1, rand2, secs);
    println!("{}", uuid);
    // will be 128bits of data in 32 byte
    debug_assert_eq!(32, uuid.chars().count());

    std::fs::write(MACHINE_ID_FILE_PATH, uuid)
}

fn get_machine_id() -> Result<String, std::io::Error> {
    if !std::path::PathBuf::from(MACHINE_ID_FILE_PATH).exists() {
        create_and_store_machine_uuid()?;
    }
    std::fs::read(MACHINE_ID_FILE_PATH).map(|vec| String::from_utf8(vec).unwrap())
}

/// Handles messages that are of the org.freedesktop.DBus.Peer interface. Returns as a bool whether the message was actually
/// of that interface and an Error if there were any while handling the message
pub fn handle_peer_message(
    msg: &Message,
    con: &mut Conn,
    timeout: client_conn::Timeout,
) -> Result<bool, crate::client_conn::Error> {
    if let Some(interface) = &msg.dynheader.interface {
        if interface.eq("org.freedesktop.DBus.Peer") {
            if let Some(member) = &msg.dynheader.member {
                match member.as_str() {
                    "Ping" => {
                        let mut reply = msg.make_response();
                        con.send_message(&mut reply, timeout)?;
                        Ok(true)
                    }
                    "GetMachineId" => {
                        let mut reply = msg.make_response();
                        reply.body.push_param(get_machine_id().unwrap()).unwrap();
                        con.send_message(&mut reply, timeout)?;
                        Ok(true)
                    }

                    // anything else is not in this interface and thus not handled here
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}
