use common::{database::Database, prelude::*};
use enrgy::{
    extractor,
    http::{headers::CONTENT_TYPE, HttpResponse},
    response::IntoResponse,
};

use crate::{
    handlers::Template,
    templates::{pages, partials, Layout, Width},
    utils,
};

pub fn index(db: extractor::Data<Database>) -> Result<impl IntoResponse, pages::Error> {
    let mut stories = db
        .index()
        .stories
        .keys()
        .map(|id| {
            utils::get_story_full(&db, id)
                .and_then(|story| partials::StoryPartial::new(id.clone(), story, None))
        })
        .collect::<Result<Vec<partials::StoryPartial>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    Ok(Template(Layout::new(
        Width::Slim,
        db.settings().theme,
        "home",
        None,
        pages::Index::new(stories),
    )))
}

pub fn favicon() -> HttpResponse {
    HttpResponse::ok()
        .header(CONTENT_TYPE, "image/x-icon")
        .body(common::ICON)
}
