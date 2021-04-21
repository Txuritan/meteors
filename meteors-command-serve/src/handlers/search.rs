use {
    crate::{
        router::{Context, Response},
        search, utils,
        views::{IndexPage, Layout, StoryCard},
    },
    common::{database::Database, prelude::*},
};

pub fn search(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or_else(|| "light".into());

    let query = ctx
        .query("search")
        .ok_or_else(|| anyhow!("search query string not found in url"))?;

    let ids = search::search(db, &query);

    let query = ctx.rebuild_query();

    let stories = ids
        .iter()
        .map(|id| {
            utils::get_story_full(db, id)
                .and_then(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    let body = Layout::new("search", theme, query, IndexPage::new(stories));

    Ok(crate::res!(200; body))
}
