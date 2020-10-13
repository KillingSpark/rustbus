#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate rustbus;

fuzz_target!(|data: &[u8]| {
    rustbus::wire::unmarshal::unmarshal_header(data, 0);
});
