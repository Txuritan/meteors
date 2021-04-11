#[cfg(test)]
mod tests;

pub mod epub;
pub mod gztar;
pub mod html;

use {
    crate::{models::proto::Rating, prelude::*},
    std::{ffi::OsStr, fs::DirEntry},
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

pub fn handle(entry: &DirEntry) -> Result<()> {
    let path = entry.path();
    let ext = path.extension().and_then(OsStr::to_str);

    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

    match ext {
        Some("epub") => {}
        Some("html") => {}
        Some("gztar") => {}
        Some(ext) => {}
        None => {}
    }

    Ok(())
}
