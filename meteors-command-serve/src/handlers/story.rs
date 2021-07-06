use {
    crate::{
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    enrgy::{web, HttpResponse},
};

pub fn story(
    db: web::Data<Database>,
    id: web::Param<"id">,
    index: web::Param<"chapter">,
) -> HttpResponse {
    utils::wrap(|| {
        let index: usize = index.parse().map_err(anyhow::Error::from)?;

        let (_, story) = utils::get_story_full(&*db, &id)?;
        let chapter = db.get_chapter_body(&id, index)?;

        let body = Layout::new(
            Width::Slim,
            db.settings().theme,
            story.info.title.clone(),
            None,
            pages::Chapter::new(partials::StoryCard::new(&id, story, None)?, &chapter, index),
        );

        Ok(crate::res!(200; body))
    })
}
