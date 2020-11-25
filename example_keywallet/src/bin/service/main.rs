//! This servers as a testing ground for rustbus. It implements the secret-service API from freedesktop.org (https://specifications.freedesktop.org/secret-service/latest/).
//! Note though that this is not meant as a real secret-service you should use, it will likely be very insecure. This is just to have a realworld
//! usecase to validate the existing codebase and new ideas
use std::collections::HashMap;

use rustbus::connection::dispatch_conn::DispatchConn;
use rustbus::connection::dispatch_conn::HandleEnvironment;
use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::connection::get_session_bus_path;
use rustbus::connection::ll_conn::Conn;
use rustbus::message_builder::MarshalledMessage;
use rustbus::wire::marshal::traits::ObjectPath;
use rustbus::wire::unmarshal::traits::Variant;

use example_keywallet::messages;

mod service;
struct Context {
    service: service::SecretService,
}
type MyHandleEnv<'a, 'b> = HandleEnvironment<'a, &'b mut Context, ()>;

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

fn service_handler(
    ctx: &mut &mut Context,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
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
            match msg
                .dynheader
                .member
                .as_ref()
                .expect("NO MEMBER :(")
                .as_str()
            {
                "OpenSession" => {
                    let (alg, _input) = msg
                        .body
                        .parser()
                        .get2::<&str, Variant>()
                        .expect("Types did not match!");
                    println!("Open Session with alg: {}", alg);

                    ctx.service.open_session(alg).unwrap();
                    let mut resp = msg.dynheader.make_response();
                    resp.body.push_variant(()).unwrap();
                    resp.body
                        .push_param(ObjectPath::new("/A/B/C").unwrap())
                        .unwrap();
                    Ok(Some(resp))
                }
                "CreateCollection" => {
                    let (props, alias): (HashMap<&str, Variant>, &str) =
                        msg.body.parser().get2().expect("Types did not match!");
                    println!(
                        "Create collection with props: {:?} and alias: {}",
                        props, alias
                    );

                    ctx.service.create_collection("ABCD").unwrap();
                    let mut resp = msg.dynheader.make_response();
                    resp.body
                        .push_param(ObjectPath::new("/A/B/C").unwrap())
                        .unwrap();
                    resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
                    Ok(Some(resp))
                }
                "SearchItems" => {
                    let attrs: HashMap<&str, &str> =
                        msg.body.parser().get().expect("Types did not match!");
                    println!("Search items with attrs: {:?}", attrs);

                    let mut resp = msg.dynheader.make_response();
                    resp.body
                        .push_param(&[ObjectPath::new("/A/B/C").unwrap()][..])
                        .unwrap();
                    resp.body
                        .push_param(&[ObjectPath::new("/A/B/D").unwrap()][..])
                        .unwrap();
                    Ok(Some(resp))
                }
                "Unlock" => {
                    let objects: Vec<ObjectPath> =
                        msg.body.parser().get().expect("Types did not match!");
                    println!("Unlock objects: {:?}", objects);

                    let mut resp = msg.dynheader.make_response();
                    resp.body
                        .push_param(&[ObjectPath::new("/A/B/C").unwrap()][..])
                        .unwrap();
                    resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
                    Ok(Some(resp))
                }
                "Lock" => {
                    let objects: Vec<ObjectPath> =
                        msg.body.parser().get().expect("Types did not match!");
                    println!("Lock objects: {:?}", objects);

                    let mut resp = msg.dynheader.make_response();
                    resp.body
                        .push_param(&[ObjectPath::new("/A/B/C").unwrap()][..])
                        .unwrap();
                    resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
                    Ok(Some(resp))
                }
                "GetSecrets" => {
                    let (items, session): (Vec<ObjectPath>, ObjectPath) =
                        msg.body.parser().get2().expect("Types did not match!");
                    println!("Get secrets: {:?} for session {:?}", items, session);

                    let mut secrets = HashMap::new();
                    secrets.insert(
                        ObjectPath::new("/A/B/C").unwrap(),
                        messages::Secret {
                            session: session.clone(),
                            params: vec![],
                            value: "very secret much info".as_bytes().to_vec(),
                            content_type: "text/plain".to_owned(),
                        },
                    );

                    let mut resp = msg.dynheader.make_response();
                    resp.body.push_param(secrets).unwrap();
                    Ok(Some(resp))
                }
                "ReadAlias" => {
                    let alias: &str = msg.body.parser().get().expect("Types did not match!");
                    println!("Read alias: {}", alias);

                    let mut resp = msg.dynheader.make_response();
                    resp.body
                        .push_param(&[ObjectPath::new("/A/B/C").unwrap()][..])
                        .unwrap();
                    Ok(Some(resp))
                }
                "SetAlias" => {
                    let (alias, object): (&str, ObjectPath) =
                        msg.body.parser().get2().expect("Types did not match!");
                    println!("Set alias for object {:?} {}", object, alias);

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
fn collection_handler(
    _ctx: &mut &mut Context,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the collection handler got called for: {:?}",
        msg.dynheader
    );
    Ok(None)
}
fn item_handler(
    _ctx: &mut &mut Context,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the item handler got called for: {:?}",
        msg.dynheader
    );
    Ok(None)
}
fn session_handler(
    _ctx: &mut &mut Context,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut MyHandleEnv,
) -> HandleResult<()> {
    println!(
        "Woohoo the session handler got called for: {:?}",
        msg.dynheader
    );
    Ok(None)
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
        "/org/freedesktop/secrets/collection/:colllection_id",
        collection_handler,
    );
    dp_con.add_handler(
        "/org/freedesktop/secrets/collection/:colllection_id/:item_id",
        item_handler,
    );
    dp_con.add_handler(
        "/org/freedesktop/secrets/session/:session_id",
        session_handler,
    );

    dp_con.run().unwrap();
}
