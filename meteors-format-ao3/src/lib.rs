#[cfg(test)]
mod tests;

pub mod epub;
pub mod gztar;
pub mod html;

use {
    common::{
        models::proto::{Range, Rating},
        prelude::*,
    },
    query::Document,
    std::convert::TryFrom,
};

#[derive(Debug, PartialEq)]
pub struct ParsedInfo {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
}

#[derive(Debug, PartialEq)]
pub struct ParsedMeta {
    pub rating: Rating,
    pub categories: Vec<String>,
    pub origins: Vec<String>,
    pub warnings: Vec<String>,
    pub pairings: Vec<String>,
    pub characters: Vec<String>,
    pub generals: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct ParsedChapters {
    pub chapters: Vec<ParsedChapter>,
}

#[derive(Debug, PartialEq)]
pub struct ParsedChapter {
    pub title: String,
    pub summary: Option<String>,
    pub start_notes: Option<Range>,
    pub content: Range,
    pub end_notes: Option<Range>,
}

#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    Epub,
    Html,
    Gztar,
}

pub fn parse(kind: FileKind, input: &str) -> Result<(ParsedInfo, ParsedMeta, ParsedChapters)> {
    match kind {
        FileKind::Epub => todo!(),
        FileKind::Html => {
            let doc = Document::try_from(input)?;

            let info = html::parse_info(&doc);
            let meta = html::parse_meta(&doc);
            let chapters = html::parse_chapters(&doc)?;

            Ok((info, meta, chapters))
        }
        FileKind::Gztar => todo!(),
    }
}
