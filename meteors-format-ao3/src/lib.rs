#[cfg(test)]
mod tests;

pub mod epub;
pub mod gztar;
pub mod html;

use {
    common::{models::Rating, prelude::*},
    flate2::read::DeflateDecoder,
    query::Document,
    std::{convert::TryFrom, ops::Range},
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
    pub start_notes: Option<Range<usize>>,
    pub content: Range<usize>,
    pub end_notes: Option<Range<usize>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    Epub,
    Html,
}

pub fn parse(kind: FileKind, bytes: Vec<u8>) -> Result<(ParsedInfo, ParsedMeta, ParsedChapters)> {
    match kind {
        FileKind::Epub => {
            let mut decoder = DeflateDecoder::new(&bytes[..]);

            todo!()
        }
        FileKind::Html => {
            let text = String::from_utf8(bytes)?;

            let doc = Document::try_from(text.as_str())?;

            let info = html::parse_info(&doc);
            let meta = html::parse_meta(&doc);
            let chapters = html::parse_chapters(&doc)?;

            Ok((info, meta, chapters))
        }
    }
}
