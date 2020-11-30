//! This servers as a testing ground for rustbus. It implements the secret-service API from freedesktop.org (https://specifications.freedesktop.org/secret-service/latest/).
//! Note though that this is not meant as a real secret-service you should use, it will likely be very insecure. This is just to have a realworld
//! usecase to validate the existing codebase and new ideas
use rustbus::connection::dispatch_conn::DispatchConn;
use rustbus::connection::dispatch_conn::HandleEnvironment;
use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::connection::get_session_bus_path;
use rustbus::connection::ll_conn::Conn;
use rustbus::message_builder::MarshalledMessage;
use rustbus::wire::marshal::traits::ObjectPath;

mod collection_interface;
mod item_interface;
mod service;
mod service_interface;
pub struct Context {
    service: service::SecretService,
}
pub type MyHandleEnv<'a, 'b> = HandleEnvironment<'a, &'b mut Context, ()>;

fn default_handler(
    _ctx: &mut &mut Context,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the default handler got called for: {:?}",
        msg.dynheader
    );
    Ok(None)
}

enum ObjectType<'a> {
    Collection(&'a str),
    Item { col: &'a str, item: &'a str },
    Session(&'a str),
}

fn get_object_type_and_id<'a>(path: &'a ObjectPath<&'a str>) -> Option<ObjectType<'a>> {
    let mut split = path.as_ref().split("/");
    let typ = split.nth(3)?;
    let id = split.nth(0)?;
    let item_id = split.nth(0);
    match typ {
        "collection" => {
            if item_id.is_some() {
                Some(ObjectType::Item {
                    col: id,
                    item: item_id.unwrap(),
                })
            } else {
                Some(ObjectType::Collection(id))
            }
        }
        "session" => Some(ObjectType::Session(id)),
        _ => None,
    }
}

fn service_handler(
    ctx: &mut &mut Context,
    matches: Matches,
    msg: &MarshalledMessage,
    env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the service handler got called for: {:?}",
        msg.dynheader
    );

    match msg
        .dynheader
        .interface
        .as_ref()
        .expect("NO INTERFACE :(")
        .as_str()
    {
        "org.freedesktop.Secret.Service" => {
            service_interface::handle_service_interface(ctx, matches, msg, env)
        }
        other => {
            println!("Unkown interface called: {}", other);
            Ok(Some(rustbus::standard_messages::unknown_method(
                &msg.dynheader,
            )))
        }
    }
}
fn collection_handler(
    ctx: &mut &mut Context,
    matches: Matches,
    msg: &MarshalledMessage,
    env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the collection handler got called for: {:?}",
        msg.dynheader
    );

    match msg
        .dynheader
        .interface
        .as_ref()
        .expect("NO INTERFACE :(")
        .as_str()
    {
        "org.freedesktop.Secret.Collection" => {
            collection_interface::handle_collection_interface(ctx, matches, msg, env)
        }
        other => {
            println!("Unkown interface called: {}", other);
            Ok(Some(rustbus::standard_messages::unknown_method(
                &msg.dynheader,
            )))
        }
    }
}
fn item_handler(
    ctx: &mut &mut Context,
    matches: Matches,
    msg: &MarshalledMessage,
    env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the item handler got called for: {:?}",
        msg.dynheader
    );

    match msg
        .dynheader
        .interface
        .as_ref()
        .expect("NO INTERFACE :(")
        .as_str()
    {
        "org.freedesktop.Secret.Item" => {
            item_interface::handle_item_interface(ctx, matches, msg, env)
        }
        other => {
            println!("Unkown interface called: {}", other);
            Ok(Some(rustbus::standard_messages::unknown_method(
                &msg.dynheader,
            )))
        }
    }
}
fn session_handler(
    ctx: &mut &mut Context,
    matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the session handler got called for: {:?}",
        msg.dynheader
    );
    let ses_id = matches
        .matches
        .get(":collection_id")
        .expect("Called session interface without a match on \":session_id\"");
    match msg
        .dynheader
        .interface
        .as_ref()
        .expect("NO INTERFACE :(")
        .as_str()
    {
        "org.freedesktop.Secret.Session" => {
            match msg
                .dynheader
                .member
                .as_ref()
                .expect("NO MEMBER :(")
                .as_str()
            {
                "Close" => {
                    ctx.service.close_session(ses_id).unwrap();
                    Ok(None)
                }
                other => {
                    println!("Unkown method called: {}", other);
                    Ok(Some(rustbus::standard_messages::unknown_method(
                        &msg.dynheader,
                    )))
                }
            }
        }
        other => {
            println!("Unkown interface called: {}", other);
            Ok(Some(rustbus::standard_messages::unknown_method(
                &msg.dynheader,
            )))
        }
    }
}

fn main() {
    let con = Conn::connect_to_bus(get_session_bus_path().unwrap(), false).unwrap();
    let mut ctx = Context {
        service: service::SecretService::default(),
    };

    let dh = Box::new(default_handler);
    let mut dp_con = DispatchConn::new(con, &mut ctx, dh);

    let service_handler = Box::new(service_handler);
    let collection_handler = Box::new(collection_handler);
    let item_handler = Box::new(item_handler);
    let session_handler = Box::new(session_handler);
    dp_con.add_handler("/org/freedesktop/secrets", service_handler);
    dp_con.add_handler(
        "/org/freedesktop/secrets/collection/:collection_id",
        collection_handler,
    );
    dp_con.add_handler(
        "/org/freedesktop/secrets/collection/:collection_id/:item_id",
        item_handler,
    );
    dp_con.add_handler(
        "/org/freedesktop/secrets/session/:session_id",
        session_handler,
    );

    dp_con.run().unwrap();
}
