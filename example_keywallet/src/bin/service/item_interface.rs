use example_keywallet::messages;
use rustbus::connection::dispatch_conn::HandleResult;
use rustbus::connection::dispatch_conn::Matches;
use rustbus::message_builder::MarshalledMessage;
use rustbus::wire::marshal::traits::ObjectPath;

pub fn handle_item_interface(
    ctx: &mut &mut super::Context,
    matches: Matches,
    msg: &MarshalledMessage,
    _env: &mut super::MyHandleEnv,
) -> HandleResult<()> {
    let col_id = matches
        .matches
        .get(":collection_id")
        .expect("Called collection interface without a match on \":collection_id\"");
    let item_id = matches
        .matches
        .get(":item_id")
        .expect("Called item interface without a match on \":item_id\"");

    match msg
        .dynheader
        .member
        .as_ref()
        .expect("NO MEMBER :(")
        .as_str()
    {
        "Delete" => {
            println!("Delete item: {:?}", msg.dynheader.object.as_ref().unwrap());

            ctx.service.delete_item(col_id, item_id).unwrap();

            let mut resp = msg.dynheader.make_response();
            resp.body.push_param(ObjectPath::new("/").unwrap()).unwrap();
            Ok(Some(resp))
        }

        "GetSecret" => {
            println!(
                "Get secret from item: {:?}",
                msg.dynheader.object.as_ref().unwrap()
            );

            let session: ObjectPath<&str> = msg.body.parser().get().expect("Types did not match");
            let secret = ctx.service.get_secret(col_id, item_id).unwrap();
            let mut resp = msg.dynheader.make_response();
            resp.body
                .push_param(messages::Secret {
                    session: session.to_owned(),
                    params: secret.params.clone(),
                    value: secret.value.clone(),
                    content_type: secret.content_type.clone(),
                })
                .unwrap();
            Ok(Some(resp))
        }

        "SetSecret" => {
            println!(
                "Set secret for item: {:?}",
                msg.dynheader.object.as_ref().unwrap()
            );

            let secret: messages::Secret = msg.body.parser().get().expect("Types did not match");
            ctx.service
                .set_secret(
                    col_id,
                    item_id,
                    example_keywallet::Secret {
                        value: secret.value,
                        params: secret.params,
                        content_type: secret.content_type,
                    },
                )
                .unwrap();
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
