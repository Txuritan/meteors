use {
    crate::{
        router::{Context, Response},
        search,
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
};

pub fn search(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx
        .state
        .read()
        .map_err(|err| anyhow!("Unable to get read lock on the database: {:?}", err))?;

    let query = ctx
        .query("search")
        .ok_or_else(|| anyhow!("search query string not found in url"))?;

    let ids = search::search(&*db, &query);

    let query = ctx.rebuild_query();

    let mut stories = ids
        .iter()
        .map(|id| {
            utils::get_story_full(&*db, id)
                .and_then(|(id, story)| partials::StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    let body = Layout::new(
        Width::Slim,
        db.settings().theme(),
        "search",
        query,
        pages::Index::new(stories),
    );

    Ok(crate::res!(200; body))
}

pub fn search_v2(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx
        .state
        .read()
        .map_err(|err| anyhow!("Unable to get read lock on the database: {:?}", err))?;

    let mut stories = db.index().stories.iter().collect::<Vec<_>>();

    let stats = search::search_v2(&ctx.query[..], &mut stories)
        .fill(db.index())
        .ok_or_else(|| anyhow!("Unable to fill out stats, an entity does not exist somewhere"))?;

    let query = ctx.rebuild_query();

    let mut stories = stories
        .into_iter()
        .map(|(id, _)| {
            utils::get_story_full(&*db, id)
                .and_then(|(id, story)| partials::StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    let body = Layout::new(
        Width::Wide,
        db.settings().theme(),
        "search",
        query,
        pages::Search::new(stories, stats),
    );

    Ok(crate::res!(200; body))
}
