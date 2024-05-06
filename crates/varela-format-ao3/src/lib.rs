#[cfg(test)]
mod tests;

pub mod epub;
pub mod html;

use {
    common::{
        models::{FileKind, Rating},
        prelude::*,
    },
    query::Document,
    std::{convert::TryFrom, io::Cursor, ops::Range, path::Path},
};

#[derive(PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ParsedInfo {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ParsedMeta {
    pub rating: Rating,
    pub categories: Vec<String>,
    pub origins: Vec<String>,
    pub warnings: Vec<String>,
    pub pairings: Vec<String>,
    pub characters: Vec<String>,
    pub generals: Vec<String>,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ParsedChapters {
    pub chapters: Vec<ParsedChapter>,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ParsedChapter {
    pub title: String,
    pub summary: Option<String>,
    pub start_notes: Option<Range<usize>>,
    pub content: Range<usize>,
    pub end_notes: Option<Range<usize>>,
}

pub fn parse_epub(path: &Path) -> Result<(ParsedInfo, ParsedMeta, ParsedChapters)> {
    todo!()
}

#[inline]
pub fn parse_html(text: &str) -> Result<(ParsedInfo, ParsedMeta, ParsedChapters)> {
    let doc = Document::try_from(text)?;

    let info = html::parse_info(&doc);
    let meta = html::parse_meta(&doc);
    let chapters = html::parse_chapters(&doc)?;

    Ok((info, meta, chapters))
}
