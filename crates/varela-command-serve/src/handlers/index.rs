use crate::{
    templates::{pages, partials, Layout, Width},
    utils,
};
use common::{database::Database, prelude::*};
use enrgy::{http::headers::CONTENT_TYPE, http::HttpResponse, web};

pub fn index(db: web::Data<Database>) -> HttpResponse {
    utils::wrap(|| {
        let mut stories = db
            .index()
            .stories
            .keys()
            .map(|id| {
                utils::get_story_full(&*db, id)
                    .and_then(|story| partials::StoryPartial::new(id, story, None))
            })
            .collect::<Result<Vec<partials::StoryPartial<'_>>>>()?;

        stories.sort_by(|a, b| a.title().cmp(b.title()));

        let body = Layout::new(
            Width::Slim,
            db.settings().theme,
            "home",
            None,
            pages::Index::new(stories),
        );

        Ok(crate::res!(200; body))
    })
}

pub fn favicon() -> HttpResponse {
    HttpResponse::ok()
        .header(CONTENT_TYPE, "image/x-icon")
        .body(Vec::from(common::ICON))
}
