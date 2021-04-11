use {
    crate::{
        format::{ParsedInfo, ParsedMeta},
        models::proto::Rating,
        prelude::*,
    },
    query::{Document, Dom, Node},
};

/// Used to get the parent element `p` to be able to get the story id from its children `a`s
#[query::selector]
static ROOT_MESSAGE: &str = "html > body > #preface > .message";

/// Selects the chapter title
#[query::selector]
static CHAPTER_META_TITLE: &str = "html > body > #chapters > .meta.group > h2";

/// Selects the chapter summary
#[query::selector]
static CHAPTER_META_SUMMARY: &str = "html > body > #chapters > .meta.group > blockquote > p";

pub fn parse<'input>(dom: Dom<'input>) -> Result<()> {
    Ok(())
}

pub fn parse_info<'input>(doc: &Document<'input>) -> ParsedInfo<'input> {
    #[query::selector]
    static META_TITLE: &str = "html > body > #preface > .meta > h1";

    /// Selects the `byline` to get the story authors
    #[query::selector]
    static META_BYLINE: &str = "html > body > #preface > .meta > .byline > a[rel=author]";

    /// Selects the stories summary and notes
    #[query::selector]
    static META_SUMMARY: &str = "html > body > #preface > .meta > blockquote .userstuff";

    let title = doc
        .select(&META_TITLE)
        .and_then(Node::into_text)
        .unwrap_or("");

    let summary = doc
        .select(&META_SUMMARY)
        .and_then(Node::into_text)
        .unwrap_or("");

    let authors = {
        let authors: Option<Vec<&'input str>> = doc
            .select_all(&META_BYLINE)
            .into_iter()
            .map(Node::into_text)
            .collect();

        match authors {
            Some(mut authors) => {
                if authors.is_empty() {
                    authors.push("Anonymous");
                }

                authors
            }
            None => {
                vec!["Anonymous"]
            }
        }
    };

    ParsedInfo {
        title,
        authors,
        summary,
    }
}

pub fn parse_meta<'input>(doc: &Document<'input>) -> ParsedMeta<'input> {
    #[query::selector]
    static META_TAGS_DT: &str = "html > body > #preface > .meta > .tags > dt";

    #[query::selector]
    static META_TAGS_DF: &str = "html > body > #preface > .meta > .tags > dd";

    let detail_names = doc.select_all(&META_TAGS_DT);
    let detail_definitions = doc.select_all(&META_TAGS_DF);

    let mut rating = Rating::Unknown;

    let mut categories = Vec::new();

    let mut origins = Vec::new();

    let mut warnings = Vec::new();
    let mut pairings = Vec::new();
    let mut characters = Vec::new();
    let mut generals = Vec::new();

    let nodes = detail_names.into_iter().zip(detail_definitions.into_iter());

    for (detail_names, detail_definition) in nodes {
        let text = match detail_names.get_text().map(|text| text.trim()) {
            Some(text) => text.trim(),
            None => continue,
        };

        let part = match text {
            "Rating:" => {
                let text = detail_definition
                    .children
                    .get(0)
                    .and_then(|node| node.get_text().map(|text| text.trim()));

                match text {
                    Some("Explicit") => rating = Rating::Explicit,
                    Some("Mature") => rating = Rating::Mature,
                    Some("Teen And Up Audiences") => rating = Rating::Teen,
                    Some("Not Rated") => rating = Rating::NotRated,
                    _ => (),
                }

                None
            }
            "Archive Warning:" => Some((&mut warnings, &detail_definition)),
            "Category:" => Some((&mut categories, &detail_definition)),
            "Fandom:" => Some((&mut origins, &detail_definition)),
            "Relationship:" => Some((&mut pairings, &detail_definition)),
            "Character:" => Some((&mut characters, &detail_definition)),
            "Additional Tags:" => Some((&mut generals, &detail_definition)),
            _ => None,
        };

        if let Some((list, node)) = part {
            for child in &node.children {
                if let Some(text) = child.get_text() {
                    list.push(text);
                }
            }
        }
    }

    ParsedMeta {
        rating,
        categories,
        origins,
        warnings,
        pairings,
        characters,
        generals,
    }
}
