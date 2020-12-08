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
    msg.unmarshall_all().ok();
});
