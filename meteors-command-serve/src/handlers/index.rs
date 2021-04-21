use {
    crate::{
        router::{Context, Response},
        utils,
        views::{IndexPage, Layout, StoryCard},
    },
    common::{database::Database, prelude::*},
};

pub fn index(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or_else(|| "light".into());

    let query = ctx.rebuild_query();

    let stories = db
        .index
        .stories
        .keys()
        .map(|id| {
            utils::get_story_full(db, id)
                .and_then(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<StoryCard<'_>>>>()?;

    let body = Layout::new("home", theme, query, IndexPage::new(stories));

    Ok(crate::res!(200; body))
}
