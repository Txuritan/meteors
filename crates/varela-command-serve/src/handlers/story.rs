use common::{database::Database, models::Id, prelude::*};
use enrgy::{http::HttpResponse, web};

use crate::{
    templates::{pages, partials, Layout, Width},
    utils,
};

pub fn story(
    db: web::Data<Database>,
    id: web::ParseParam<"id", Id>,
    index: web::Param<"chapter">,
) -> HttpResponse {
    utils::wrap(|| {
        let index: usize = index.parse().map_err(anyhow::Error::from)?;

        let story = utils::get_story_full(&*db, &id)?;
        let chapter = db.get_chapter_body(&id, index)?;

        let body = Layout::new(
            Width::Slim,
            db.settings().theme,
            story.info.title.clone(),
            None,
            pages::Chapter::new(
                partials::StoryPartial::new(&id, story, None)?,
                &chapter,
                index,
            ),
        );

        Ok(crate::res!(200; body))
    })
}
