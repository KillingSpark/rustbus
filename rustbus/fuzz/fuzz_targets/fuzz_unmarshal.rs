#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate rustbus;

fuzz_target!(|data: &[u8]| {
    let (hdrbytes, header) = match rustbus::wire::unmarshal::unmarshal_header(data, 0) {
        Ok(head) => head,
        Err(_) => return,
    };
    let (dynhdrbytes, dynheader) =
        match rustbus::wire::unmarshal::unmarshal_dynamic_header(&header, data, hdrbytes) {
            Ok(head) => head,
            Err(_) => return,
        };

    let (_bytes_used, msg) = match rustbus::wire::unmarshal::unmarshal_next_message(
        &header,
        dynheader,
        data,
        hdrbytes + dynhdrbytes,
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
        .get::<rustbus::wire::marshal::traits::ObjectPath<&str>>()
        .ok();
    msg.body
        .parser()
        .get::<rustbus::wire::marshal::traits::SignatureWrapper>()
        .ok();
    msg.body.parser().get::<u64>().ok();
    msg.body.parser().get::<u32>().ok();
    msg.body.parser().get::<u16>().ok();
    msg.body.parser().get::<i64>().ok();
    msg.body.parser().get::<i32>().ok();
    msg.body.parser().get::<i16>().ok();
    msg.body.parser().get::<u8>().ok();

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
}
