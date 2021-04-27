use {
    crate::{
        router::{Context, Response},
        utils,
        views::{IndexPage, Layout, StoryCard},
    },
    common::{database::Database, prelude::*},
};

pub fn index(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx
        .state
        .read()
        .map_err(|err| anyhow!("Unable to get read lock on the database: {:?}", err))?;

    let query = ctx.rebuild_query();

    let mut stories = db
        .index()
        .stories
        .keys()
        .map(|id| {
            utils::get_story_full(&*db, id)
                .and_then(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<StoryCard<'_>>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    let body = Layout::new(
        "home",
        db.settings().theme(),
        query,
        IndexPage::new(stories),
    );

    Ok(crate::res!(200; body))
}
