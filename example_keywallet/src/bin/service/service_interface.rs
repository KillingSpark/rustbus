use std::collections::HashMap;

use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::message_builder::MarshalledMessage;
use rustbus::wire::marshal::traits::ObjectPath;
use rustbus::wire::unmarshal::traits::Variant;

use super::service;
use example_keywallet::messages;

pub fn handle_service_interface(
    ctx: &mut &mut super::Context,
    _matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut super::MyHandleEnv,
) -> HandleResult<()> {
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
            let attrs: HashMap<&str, &str> = msg.body.parser().get().expect("Types did not match!");
            println!("Search items with attrs: {:?}", attrs);

            let attrs = attrs
                .into_iter()
                .map(|(name, value)| example_keywallet::LookupAttribute {
                    name: name.to_owned(),
                    value: value.to_owned(),
                })
                .collect::<Vec<_>>();
            let item_ids = ctx.service.search_items(&attrs);

            let owned_paths: Vec<(String, &service::Item)> = item_ids
                .into_iter()
                .map(|(col, item)| {
                    (
                        format!("/org/freedesktop/secrets/collection/{}/{}", col, item.id),
                        item,
                    )
                })
                .collect();

            let unlocked_object_paths: Vec<ObjectPath<&str>> = owned_paths
                .iter()
                .filter(|(_, item)| {
                    matches!(item.lock_state, example_keywallet::LockState::Unlocked)
                })
                .map(|(path, _)| ObjectPath::new(path.as_str()).unwrap())
                .collect();
            let locked_object_paths: Vec<ObjectPath<&str>> = owned_paths
                .iter()
                .filter(|(_, item)| matches!(item.lock_state, example_keywallet::LockState::Locked))
                .map(|(path, _)| ObjectPath::new(path.as_str()).unwrap())
                .collect();

            let mut resp = msg.dynheader.make_response();
            resp.body
                .push_param(unlocked_object_paths.as_slice())
                .unwrap();
            resp.body
                .push_param(locked_object_paths.as_slice())
                .unwrap();
            Ok(Some(resp))
        }
        "Unlock" => {
            let objects: Vec<ObjectPath<&str>> =
                msg.body.parser().get().expect("Types did not match!");
            println!("Unlock objects: {:?}", objects);

            for object in &objects {
                if let Some(object) = super::get_object_type_and_id(object) {
                    match object {
                        super::ObjectType::Collection(id) => {
                            ctx.service.unlock_collection(id).unwrap()
                        }
                        super::ObjectType::Item { col, item } => {
                            ctx.service.unlock_item(col, item).unwrap()
                        }
                        super::ObjectType::Session(_) => println!("Tried to unlock session O_o"),
                    }
                }
            }

            let mut resp = msg.dynheader.make_response();
            resp.body.push_param(objects.as_slice()).unwrap();
            resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
            Ok(Some(resp))
        }
        "Lock" => {
            let objects: Vec<ObjectPath<&str>> =
                msg.body.parser().get().expect("Types did not match!");
            println!("Lock objects: {:?}", objects);

            for object in &objects {
                if let Some(object) = super::get_object_type_and_id(object) {
                    match object {
                        super::ObjectType::Collection(id) => {
                            ctx.service.lock_collection(id).unwrap()
                        }
                        super::ObjectType::Item { col, item } => {
                            ctx.service.lock_item(col, item).unwrap()
                        }
                        super::ObjectType::Session(_) => println!("Tried to unlock session O_o"),
                    }
                }
            }

            let mut resp = msg.dynheader.make_response();
            resp.body.push_param(objects.as_slice()).unwrap();
            resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
            Ok(Some(resp))
        }
        "GetSecrets" => {
            let (items, session): (Vec<ObjectPath<&str>>, ObjectPath<&str>) =
                msg.body.parser().get2().expect("Types did not match!");
            println!("Get secrets: {:?} for session {:?}", items, session);

            let mut secrets: HashMap<ObjectPath<String>, messages::Secret> = HashMap::new();
            for item in &items {
                if let Some(object) = super::get_object_type_and_id(item) {
                    match object {
                        super::ObjectType::Collection(_) => {
                            println!("Tried to get a secret from a collection object O_o")
                        }
                        super::ObjectType::Item { col, item: item_id } => {
                            let secret = ctx.service.get_secret(col, item_id).unwrap();
                            secrets.insert(
                                item.to_owned(),
                                messages::Secret {
                                    session: session.to_owned(),
                                    params: secret.params.clone(),
                                    value: secret.value.clone(),
                                    content_type: secret.content_type.clone(),
                                },
                            );
                        }
                        super::ObjectType::Session(_) => println!("Tried to unlock session O_o"),
                    }
                }
            }

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
            let (alias, object): (&str, ObjectPath<&str>) =
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
