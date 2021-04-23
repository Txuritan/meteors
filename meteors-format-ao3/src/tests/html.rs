use {
    crate::{
        html::{parse_chapters, parse_info, parse_meta},
        ParsedChapter, ParsedChapters, ParsedInfo, ParsedMeta,
    },
    common::models::proto::{Range, story::meta::Rating},
    query::Document,
    std::convert::TryFrom as _,
};

static DATA_MULTIPLE: &str = include_str!("./Disrupt.html");

#[test]
fn test_parse_info_multiple() {
    let doc = Document::try_from(DATA_MULTIPLE).expect("BUG: This file should always be parsable");

    let left = ParsedInfo {
        title: "Disrupt".to_string(),
        authors: vec!["testy".to_string()],
        summary: "<p>Unicorn et Retro adipisicing yr, nulla disrupt laboris austin.</p>\n<p> </p>\n<p>  <b>Please do not delete! We may need this for further download testing.</b></p>".to_string(),
    };
    let right = parse_info(&doc);

    assert_eq!(left, right);
}

#[test]
fn test_parse_meta_multiple() {
    let doc = Document::try_from(DATA_MULTIPLE).expect("BUG: This file should always be parsable");

    let left = ParsedMeta {
        rating: Rating::Teen,
        categories: vec![],
        origins: vec!["Testing".to_string()],
        warnings: vec!["No Archive Warnings Apply".to_string()],
        pairings: vec![],
        characters: vec![],
        generals: vec![],
    };
    let right = parse_meta(&doc);

    assert_eq!(left, right);
}

#[test]
fn test_parse_chapters_multiple() {
    let doc = Document::try_from(DATA_MULTIPLE).expect("BUG: This file should always be parsable");

    let left = ParsedChapters { chapters: vec![
        ParsedChapter {
            title: "Small Batch Hashtag".to_string(),
            summary: Some("<p>Trust fund hot chicken elit blog, williamsburg semiotics asymmetrical franzen church-key portland. Meh keytar iceland semiotics, portland asymmetrical cray godard venmo forage qui consectetur cillum adipisicing</p>".to_string()),
            start_notes: Some(Range::new(3112, 3349)),
            content: Range::new(3541, 19287),
            end_notes: Some(Range::new(19419, 19430)),
        },
        ParsedChapter {
            title: "Try-hard Brunch".to_string(),
            summary: Some("<p>Helvetica bread everyday.</p>".to_string()),
            start_notes: None,
            content: Range::new(19837, 41637),
            end_notes: Some(Range::new(41769, 41905)),
        },
    ] };
    let right = parse_chapters(&doc).unwrap();

    assert_eq!(left, right);
}

static DATA_SINGLE: &str = include_str!("./A Work To Test Downloads.html");

#[test]
fn test_parse_info_single() {
    let doc = Document::try_from(DATA_SINGLE).expect("BUG: This file should always be parsable");

    let left = ParsedInfo {
        title: "A Work To Test Downloads Again".to_string(),
        authors: vec!["testy".to_string()],
        summary: "<p>This is a new work for a test.</p>".to_string(),
    };
    let right = parse_info(&doc);

    assert_eq!(left, right);
}

#[test]
fn test_parse_meta_single() {
    let doc = Document::try_from(DATA_SINGLE).expect("BUG: This file should always be parsable");

    let left = ParsedMeta {
        rating: Rating::General,
        categories: vec![],
        origins: vec!["Testing".to_string()],
        warnings: vec!["No Archive Warnings Apply".to_string()],
        pairings: vec![],
        characters: vec![],
        generals: vec![],
    };
    let right = parse_meta(&doc);

    assert_eq!(left, right);
}

#[test]
fn test_parse_chapters_single() {
    let doc = Document::try_from(DATA_SINGLE).expect("BUG: This file should always be parsable");

    let left = ParsedChapters {
        chapters: vec![ParsedChapter {
            title: "A Work To Test Downloads Again".to_string(),
            summary: None,
            start_notes: None,
            content: Range::new(2265, 18822),
            end_notes: None,
        }],
    };
    let right = parse_chapters(&doc).unwrap();

    assert_eq!(left, right);
}
