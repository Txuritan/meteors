fn main() {
    afl::fuzz!(|data: &[u8]| {
        let _ = enrgy::http::HttpRequest::fuzz_parse_reader(&mut std::io::Cursor::new(data));
    });
}
