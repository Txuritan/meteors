use {
    crate::{
        router::{Context, Response},
        utils,
        views::{ChapterPage, Layout, StoryCard},
    },
    common::{database::Database, prelude::*},
};

pub fn story(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx
        .state
        .read()
        .map_err(|err| anyhow!("Unable to get read lock on the database: {:?}", err))?;

    let theme = ctx.query("theme").unwrap_or_else(|| "light".into());

    let id = ctx
        .param("id")
        .map(String::from)
        .ok_or_else(|| anyhow!("no story id was found is the request uri"))?;
    let index: usize = ctx
        .param("chapter")
        .ok_or_else(|| anyhow!("no story id was found is the request uri"))
        .and_then(|s| s.parse().map_err(anyhow::Error::from))?;

    let (_, story) = utils::get_story_full(&*db, &id)?;
    let chapter = db.get_chapter_body(&id, index)?;

    let query = ctx.rebuild_query();

    let body = Layout::new(
        story.info.title.clone(),
        theme,
        query.clone(),
        ChapterPage::new(
            StoryCard::new(&id, story, query.clone())?,
            &chapter,
            index,
            query,
        ),
    );

    Ok(crate::res!(200; body))
}
