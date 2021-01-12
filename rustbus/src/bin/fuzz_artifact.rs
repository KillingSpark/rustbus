//! This runs the same code as the unmarshal fuzzing. It is just a helper to debug crahes/timeouts easier

use std::io::Read;

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

    let (hdrbytes, header) = match rustbus::wire::unmarshal::unmarshal_header(data, 0) {
        Ok(head) => head,
        Err(_) => return,
    };

    println!("Header: {:?}", header);

    let (dynhdrbytes, dynheader) =
        match rustbus::wire::unmarshal::unmarshal_dynamic_header(&header, data, hdrbytes) {
            Ok(head) => head,
            Err(_) => return,
        };

    println!("Dynheader: {:?}", dynheader);

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
}
