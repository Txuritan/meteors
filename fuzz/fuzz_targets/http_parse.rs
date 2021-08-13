#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = enrgy::http::HttpRequest::fuzz_parse_reader(&mut std::io::Cursor::new(data));
});
