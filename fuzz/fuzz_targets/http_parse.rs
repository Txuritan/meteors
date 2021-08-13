#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|mut data: &[u8]| {
    let _ = enrgy::http::HttpRequest::fuzz_parse_reader(&mut data);
});
