use std::str::FromStr;

use common::{
    database::Database,
    models::{Existing, FileKind},
    prelude::*,
};
use enrgy::{extractor, http::headers::CONTENT_TYPE, http::HttpResponse, response::IntoResponse};

use crate::{
    templates::pages::{self, opds::OpdsFeed},
    utils,
};

pub enum CatalogFormat {
    Atom, // opds+1.2
    Html,
    Json, // opds+2.0
}

impl FromStr for CatalogFormat {
    type Err = CatalogFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "atom" | "xml" => Ok(CatalogFormat::Atom),
            "html" => Ok(CatalogFormat::Html),
            "json" => Ok(CatalogFormat::Json),
            ext => Err(CatalogFormatError(vfmt::format!(
                "Unknown catalog format: {}",
                ext
            ))),
        }
    }
}

pub struct CatalogFormatError(String);

impl vfmt::uDebug for CatalogFormatError {
    fn fmt<W>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: vfmt::uWrite + ?Sized,
    {
        f.write_str(&self.0)
    }
}

pub fn catalog(
    db: extractor::Data<Database>,
    _ext: extractor::ParseParam<"ext", CatalogFormat>,
) -> Result<impl IntoResponse, pages::Error> {
    let mut stories = db
        .index()
        .stories
        .iter()
        .filter(|(_, s)| s.info.kind == FileKind::Epub)
        .map(|(id, _)| utils::get_story_full(&db, id).map(|story| Existing::new(id.clone(), story)))
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.info.updated.cmp(&b.info.updated));

    let updated = stories
        .first()
        .map(|story| story.info.updated.clone())
        .unwrap_or_else(|| humantime::format_rfc3339(std::time::SystemTime::now()).to_string());

    Ok(HttpResponse::ok()
        .header(CONTENT_TYPE, "application/atom+xml")
        .body(::opal::Template::render_into_string(OpdsFeed::new(
            updated, stories,
        ))?))
}
