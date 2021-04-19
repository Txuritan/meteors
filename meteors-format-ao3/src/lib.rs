#[cfg(test)]
mod tests;

pub mod epub;
pub mod gztar;
pub mod html;

use {
    common::{models::proto::Rating, prelude::*},
    query::{Document, Span},
    std::convert::TryFrom,
};

#[derive(Debug, PartialEq)]
pub struct ParsedInfo<'input> {
    pub title: &'input str,
    pub authors: Vec<&'input str>,
    pub summary: &'input str,
}

#[derive(Debug, PartialEq)]
pub struct ParsedMeta<'input> {
    pub rating: Rating,
    pub categories: Vec<&'input str>,
    pub origins: Vec<&'input str>,
    pub warnings: Vec<&'input str>,
    pub pairings: Vec<&'input str>,
    pub characters: Vec<&'input str>,
    pub generals: Vec<&'input str>,
}

#[derive(Debug, PartialEq)]
pub struct ParsedChapters<'input> {
    pub chapters: Vec<ParsedChapter<'input>>,
}

#[derive(Debug, PartialEq)]
pub struct ParsedChapter<'input> {
    pub title: &'input str,
    pub summary: Option<&'input str>,
    pub start_notes: Option<Span<'input>>,
    pub content: Span<'input>,
    pub end_notes: Option<Span<'input>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    Epub,
    Html,
    Gztar,
}

pub fn parse(
    kind: FileKind,
    input: &str,
) -> Result<(ParsedInfo<'_>, ParsedMeta<'_>, ParsedChapters<'_>)> {
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
