fn main() {
    let cur = std::env::current_dir().unwrap();
    let crashes = cur.join("fuzz").join("fuzz_output").join("default").join("crashes");

    for entry in std::fs::read_dir(&crashes).unwrap() {
        let entry = entry.unwrap();

        if entry.file_name() != std::ffi::OsStr::new("README.txt").to_os_string() {
            println!("running crash: {:?}", entry.file_name());

            let bytes = std::fs::read(entry.path()).unwrap();

            eprintln!("{:?}", enrgy::HttpRequest::parse_reader(&mut std::io::Cursor::new(bytes)));
        }
    }
}
