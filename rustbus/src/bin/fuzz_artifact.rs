//! This runs the same code as the unmarshal fuzzing. It is just a helper to debug crahes/timeouts easier

use std::io::Read;

use rustbus::wire::unmarshal_context::Cursor;

fn main() {
    for path in std::env::args().skip(1) {
        println!("Run artifact: {}", path);

        run_artifact(&path);
    }
}

fn run_artifact(path: &str) {
    let mut file = std::fs::File::open(path).unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();
    let data = &data;

    let mut cursor = Cursor::new(data);
    let Ok(header) = rustbus::wire::unmarshal::unmarshal_header(&mut cursor) else {
        return;
    };

    println!("Header: {:?}", header);

    let Ok(dynheader) = rustbus::wire::unmarshal::unmarshal_dynamic_header(&header, &mut cursor)
    else {
        return;
    };

    println!("Dynheader: {:?}", dynheader);

    let Ok(msg) = rustbus::wire::unmarshal::unmarshal_next_message(
        &header,
        dynheader,
        data.clone(),
        cursor.consumed(),
        vec![],
    ) else {
        return;
    };

    println!("Message: {:?}", msg);

    msg.unmarshall_all().ok();
}
