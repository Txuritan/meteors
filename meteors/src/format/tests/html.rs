use {
    crate::format::{html::parse_info, ParsedInfo},
    query::Document,
    std::convert::TryFrom as _,
};

static DATA: &str = include_str!("./testing backdating.html");

#[test]
fn test_parse_info() {
    let doc = Document::try_from(DATA).expect("BUG: This file should always be parsable");

    let left = ParsedInfo {
        title: "testing backdating &amp; previewing",
        authors: vec!["testy"],
        summary: "",
    };
    let right = parse_info(&doc).unwrap();

    assert_eq!(left, right);
}
