use {
    crate::{
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    tiny_http_router::{Data, HttpResponse, Param},
};

pub fn story(db: Data<Database>, id: Param<"id">, index: Param<"chapter">) -> Result<HttpResponse> {
    let index: usize = index.parse().map_err(anyhow::Error::from)?;

    let (_, story) = utils::get_story_full(&*db, &id)?;
    let chapter = db.get_chapter_body(&id, index)?;

    let body = Layout::new(
        Width::Slim,
        db.settings().theme,
        story.info.title.clone(),
        None,
        pages::Chapter::new(
            partials::StoryCard::new(&id, story, None)?,
            &chapter,
            index,
        ),
    );

    Ok(crate::res!(200; body))
}
