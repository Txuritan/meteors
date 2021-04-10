use {
    crate::{format::ParsedInfo, prelude::*},
    query::{Document, Dom, Node, NodeData},
};

/// Used to get the parent element `p` to be able to get the story id from its children `a`s
#[query::selector]
static ROOT_MESSAGE: &str = "html > body > #preface > .message";

#[query::selector]
static META_TITLE: &str = "html > body > #preface > .meta > h1";

/// Selects the `byline` to get the story authors
#[query::selector]
static META_BYLINE: &str = "html > body > #preface > .meta > .byline > a[rel=author]";

/// Selects the stories summary and notes
#[query::selector]
static META_SUMMARY: &str = "html > body > #preface > .meta > blockquote .userstuff";

/// Used to get 'tag' style information, reading in pairs of (`dt`, `dd`)
#[query::selector]
static META_TAGS: &str = "html > body > #preface > .meta > .tags";

/// Selects the chapter title
#[query::selector]
static CHAPTER_META_TITLE: &str = "html > body > #chapters > .meta.group > h2";

/// Selects the chapter summary
#[query::selector]
static CHAPTER_META_SUMMARY: &str = "html > body > #chapters > .meta.group > blockquote > p";

pub fn parse<'input>(dom: Dom<'input>) -> Result<()> {
    Ok(())
}

pub fn parse_info<'input>(doc: &Document<'input>) -> Result<ParsedInfo<'input>> {
    let title = doc.select(&META_TITLE)
        .and_then(|node| {
            node.children.get(0).map(|n| match n.data {
                NodeData::Text { contains } => contains,
                _ => "",
            })
        })
        .unwrap_or("");

    let summary = doc
        .select(&META_SUMMARY)
        .and_then(|node| {
            node.children.get(0).map(|n| match n.data {
                NodeData::Text { contains } => contains,
                _ => "",
            })
        })
        .unwrap_or("");

    let authors = {
        let mut authors: Vec<&'input str> = doc
            .select_all(&META_BYLINE)
            .into_iter()
            .map(|node| match node.children[0].data {
                NodeData::Text { contains } => contains,
                _ => unreachable!(),
            })
            .collect();

        if authors.is_empty() {
            authors.push("Anonymous");
        }

        authors
    };

    Ok(ParsedInfo {
        title,
        authors,
        summary,
    })
}

pub fn parse_meta<'input>(doc: &Document<'input>) {}
