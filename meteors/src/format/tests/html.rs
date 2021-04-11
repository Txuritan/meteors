use {
    crate::{
        format::{
            html::{parse_info, parse_meta},
            ParsedInfo, ParsedMeta,
        },
        models::proto::Rating,
    },
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
    let right = parse_info(&doc);

    assert_eq!(left, right);
}

#[test]
fn test_parse_meta() {
    let doc = Document::try_from(DATA).expect("BUG: This file should always be parsable");

    let left = ParsedMeta {
        rating: Rating::NotRated,
        categories: vec!["F/M"],
        origins: vec!["test - Fandom"],
        warnings: vec![
            "Choose Not To Use Archive Warnings",
            "Graphic Depictions Of Violence",
        ],
        pairings: vec![],
        characters: vec![],
        generals: vec![],
    };
    let right = parse_meta(&doc);

    assert_eq!(left, right);
}
