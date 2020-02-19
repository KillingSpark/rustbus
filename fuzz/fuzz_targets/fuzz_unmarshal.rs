#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate rustbus;

fuzz_target!(|data: &[u8]| {
    rustbus::unmarshal::unmarshal_header(data, 0);
});
