use common::{database::Database, prelude::*};
use enrgy::{http::HttpResponse, web};

use crate::{
    search,
    templates::{pages, partials, Layout, Width},
    utils,
};

pub fn search(
    db: web::Data<Database>,
    search: web::Query<"search">,
    query: web::RawQuery,
) -> HttpResponse {
    utils::wrap(|| {
        let ids = search::search(&*db, &search);

        let query = enrgy::http::encoding::percent::utf8_percent_encode(&query, enrgy::http::encoding::percent::CONTROLS).to_string();

        let mut stories = ids
            .iter()
            .map(|id| {
                utils::get_story_full(&*db, id)
                    .and_then(|story| partials::StoryPartial::new(id, story, Some(query.clone())))
            })
            .collect::<Result<Vec<_>>>()?;

        stories.sort_by(|a, b| a.title().cmp(b.title()));

        let body = Layout::new(
            Width::Slim,
            db.settings().theme,
            "search",
            Some(query),
            pages::Index::new(stories),
        );

        Ok(crate::res!(200; body))
    })
}

pub fn search_v2(db: web::Data<Database>, query: web::Query<"search">) -> HttpResponse {
    utils::wrap(|| {
        let mut stories = db.index().stories.iter().collect::<Vec<_>>();

        let parsed_query =
            enrgy::http::encoding::form::parse(query.trim_start_matches('?').as_bytes()).collect::<Vec<_>>();

        let stats = search::search_v2(&parsed_query[..], &mut stories)
            .fill(db.index())
            .ok_or_else(|| {
                anyhow!("Unable to fill out stats, an entity does not exist somewhere")
            })?;

        let query = enrgy::http::encoding::percent::utf8_percent_encode(&query, enrgy::http::encoding::percent::CONTROLS).to_string();

        let mut stories = stories
            .into_iter()
            .map(|(id, _)| {
                utils::get_story_full(&*db, id)
                    .and_then(|story| partials::StoryPartial::new(id, story, Some(query.clone())))
            })
            .collect::<Result<Vec<_>>>()?;

        stories.sort_by(|a, b| a.title().cmp(b.title()));

        let body = Layout::new(
            Width::Wide,
            db.settings().theme,
            "search",
            Some(query),
            pages::Search::new(stories, stats),
        );

        Ok(crate::res!(200; body))
    })
}
