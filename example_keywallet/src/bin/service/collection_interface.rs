use std::collections::HashMap;

use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::message_builder::MarshalledMessage;
use rustbus::wire::marshal::traits::ObjectPath;
use rustbus::wire::unmarshal::traits::Variant;

use super::service;
use example_keywallet::messages;

pub fn handle_collection_interface(
    ctx: &mut &mut super::Context,
    matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut super::MyHandleEnv,
) -> HandleResult<()> {
    let col_id = matches
        .matches
        .get(":collection_id")
        .expect("Called collection interface without a match on \":collection_id\"");

    match msg
        .dynheader
        .member
        .as_ref()
        .expect("NO MEMBER :(")
        .as_str()
    {
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

            let col = ctx
                .service
                .get_collection(col_id)
                .expect(&format!("Collection with ID: {} not found", col_id));
            let item_ids = col.search_items(&attrs);

            let owned_paths: Vec<(String, &service::Item)> = item_ids
                .into_iter()
                .map(|item| {
                    (
                        format!("/org/freedesktop/secrets/collection/{}/{}", col_id, item.id),
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
        "CreateItem" => {
            let (props, secret, replace): (HashMap<String, Variant>, messages::Secret, bool) =
                msg.body.parser().get3().expect("Types did not match");

            println!("Create item with props: {:?}", props);

            let new_id = ctx.service.next_id();

            let col = ctx
                .service
                .get_collection_mut(col_id)
                .expect(&format!("Collection with ID: {} not found", col_id));

            let item_id = col.create_item(new_id, &secret, &vec![], replace).unwrap();
            let path = format!("/org/freedesktop/secrets/collection/{}/{}", col_id, item_id);
            let path = ObjectPath::new(&path).unwrap();

            let mut resp = msg.dynheader.make_response();
            resp.body.push_param(path).unwrap();
            resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
            Ok(Some(resp))
        }
        "Delete" => {
            let object: ObjectPath<&str> = msg.body.parser().get().expect("Types did not match");

            println!("Delete collection {:?}", object);

            if let Some(object) = super::get_object_type_and_id(&object) {
                match object {
                    super::ObjectType::Collection(id) => {
                        ctx.service.delete_collection(id).unwrap();
                    }
                    super::ObjectType::Item { .. } => {
                        println!("Tried to delete an item through the collection API O_o")
                    }
                    super::ObjectType::Session(_) => println!("Tried to unlock session O_o"),
                }
            }

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
