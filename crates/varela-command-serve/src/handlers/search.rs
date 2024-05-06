use common::{database::Database, prelude::*};
use enrgy::{extractor, response::IntoResponse};

use crate::{
    handlers::Template,
    search,
    templates::{pages, partials, Layout, Width},
    utils,
};

pub fn search(
    db: extractor::Data<Database>,
    search: extractor::Query<"search">,
    query: extractor::RawQuery,
) -> Result<impl IntoResponse, pages::Error> {
    let ids = search::search(&db, &search);

    let query = enrgy::http::encoding::percent::utf8_percent_encode(
        &query,
        enrgy::http::encoding::percent::CONTROLS,
    )
    .to_string();

    let mut stories = ids
        .iter()
        .map(|id| {
            utils::get_story_full(&db, id).and_then(|story| {
                partials::StoryPartial::new(id.clone(), story, Some(query.clone()))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    Ok(Template(Layout::new(
        Width::Slim,
        db.settings().theme,
        "search",
        Some(query),
        pages::Index::new(stories),
    )))
}

pub fn search_v2(
    db: extractor::Data<Database>,
    query: extractor::Query<"search">,
) -> Result<impl IntoResponse, pages::Error> {
    let mut stories = db.index().stories.iter().collect::<Vec<_>>();

    let parsed_query = enrgy::http::encoding::form::parse(query.trim_start_matches('?').as_bytes())
        .collect::<Vec<_>>();

    let stats = search::search_v2(&parsed_query[..], &mut stories)
        .fill(db.index())
        .ok_or_else(|| anyhow!("Unable to fill out stats, an entity does not exist somewhere"))?;

    let query = enrgy::http::encoding::percent::utf8_percent_encode(
        &query,
        enrgy::http::encoding::percent::CONTROLS,
    )
    .to_string();

    let mut stories = stories
        .into_iter()
        .map(|(id, _)| {
            utils::get_story_full(&db, id).and_then(|story| {
                partials::StoryPartial::new(id.clone(), story, Some(query.clone()))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    Ok(Template(Layout::new(
        Width::Wide,
        db.settings().theme,
        "search",
        Some(query),
        pages::Search::new(stories, stats),
    )))
}
