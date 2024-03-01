#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate rustbus;

fuzz_target!(|data: &[u8]| {
    let mut cursor = rustbus::wire::unmarshal_context::Cursor::new(data);
    let header = match rustbus::wire::unmarshal::unmarshal_header(&mut cursor) {
        Ok(head) => head,
        Err(_) => return,
    };
    let dynheader = match rustbus::wire::unmarshal::unmarshal_dynamic_header(&header, &mut cursor) {
        Ok(head) => head,
        Err(_) => return,
    };

    let msg = match rustbus::wire::unmarshal::unmarshal_next_message(
        &header,
        dynheader,
        data.to_vec(),
        cursor.consumed(),
    ) {
        Ok(msg) => msg,
        Err(_) => return,
    };
    try_unmarhal_traits(&msg);
    msg.unmarshall_all().ok();
});

// Just try to get some types from the msg.body.parser()
// Nothing here is likely to actually return anything meaningful but it must not panic
fn try_unmarhal_traits(msg: &rustbus::message_builder::MarshalledMessage) {
    // base types
    msg.body.parser().get::<&str>().ok();
    msg.body
        .parser()
        .get::<rustbus::wire::ObjectPath<&str>>()
        .ok();
    msg.body
        .parser()
        .get::<rustbus::wire::SignatureWrapper<&str>>()
        .ok();
    msg.body.parser().get::<u64>().ok();
    msg.body.parser().get::<u32>().ok();
    msg.body.parser().get::<u16>().ok();
    msg.body.parser().get::<i64>().ok();
    msg.body.parser().get::<i32>().ok();
    msg.body.parser().get::<i16>().ok();
    msg.body.parser().get::<u8>().ok();
    msg.body
        .parser()
        .get::<rustbus::wire::unmarshal::traits::Variant>()
        .ok();

    // some collections
    use std::collections::HashMap;
    msg.body.parser().get::<(u8, u64, u8, u32)>().ok();
    msg.body.parser().get::<(u8, u64, u8, &str)>().ok();
    msg.body.parser().get::<Vec<u32>>().ok();
    msg.body.parser().get::<Vec<(u8, u32)>>().ok();
    msg.body.parser().get::<HashMap<&str, (u8, u32)>>().ok();
    msg.body
        .parser()
        .get::<HashMap<&str, Vec<(u8, u32)>>>()
        .ok();
    msg.body
        .parser()
        .get::<Vec<HashMap<&str, Vec<(u8, u32)>>>>()
        .ok();
    msg.body
        .parser()
        .get::<Vec<HashMap<&str, Vec<(u8, u32, rustbus::wire::unmarshal::traits::Variant)>>>>()
        .ok();

    // Now do the same but with get2<_>
    // any getX<_> essentially uses the same code so get2 should suffice

    // base types
    msg.body.parser().get2::<u8, &str>().ok();
    msg.body
        .parser()
        .get2::<&str, rustbus::wire::ObjectPath<&str>>()
        .ok();
    msg.body
        .parser()
        .get2::<&str, rustbus::wire::SignatureWrapper<&str>>()
        .ok();
    msg.body.parser().get2::<&str, u64>().ok();
    msg.body.parser().get2::<&str, u32>().ok();
    msg.body.parser().get2::<&str, u16>().ok();
    msg.body.parser().get2::<&str, i64>().ok();
    msg.body.parser().get2::<&str, i32>().ok();
    msg.body.parser().get2::<&str, i16>().ok();
    msg.body.parser().get2::<&str, u8>().ok();
    msg.body
        .parser()
        .get2::<&str, rustbus::wire::unmarshal::traits::Variant>()
        .ok();

    // some collections
    msg.body
        .parser()
        .get2::<(u8, u64, u8, u32), (u8, &str, u8, u32)>()
        .ok();
    msg.body.parser().get2::<Vec<Vec<u8>>, Vec<u32>>().ok();
    msg.body
        .parser()
        .get2::<HashMap<u32, i64>, Vec<(u8, u32)>>()
        .ok();
    msg.body
        .parser()
        .get2::<Vec<HashMap<&str, u8>>, HashMap<&str, (u8, u32)>>()
        .ok();
    msg.body
        .parser()
        .get2::<Vec<i32>, HashMap<&str, Vec<(u8, u32)>>>()
        .ok();
    msg.body
        .parser()
        .get2::<Vec<u32>, Vec<HashMap<&str, Vec<(u8, u32)>>>>()
        .ok();
    msg.body
        .parser()
        .get2::<Vec<(u8,u8,u32)>, Vec<HashMap<&str, Vec<(u8, u32, rustbus::wire::unmarshal::traits::Variant)>>>>()
        .ok();
}
