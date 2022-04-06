use {
    crate::{
        search,
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    enrgy::{web, http::HttpResponse},
    qstring::QString,
    std::borrow::Cow,
};

fn rebuild_query(raw_query: &web::RawQuery) -> Cow<'static, str> {
    if raw_query.is_empty() {
        Cow::from(String::new())
    } else {
        let mut parsed = QString::from(raw_query.as_str()).into_pairs();

        parsed.retain(|(k, _)| k != "search");

        Cow::from(format!("?{}", QString::new(parsed)))
    }
}

pub fn search(
    db: web::Data<Database>,
    search: web::Query<"search">,
    query: web::RawQuery,
) -> HttpResponse {
    utils::wrap(|| {
        let ids = search::search(&*db, &search);

        let query = rebuild_query(&query);

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

pub fn search_v2(db: web::Data<Database>, query: web::RawQuery) -> HttpResponse {
    utils::wrap(|| {
        let mut stories = db.index().stories.iter().collect::<Vec<_>>();

        let parsed_query =
            form_urlencoded::parse(query.trim_start_matches('?').as_bytes()).collect::<Vec<_>>();

        let stats = search::search_v2(&parsed_query[..], &mut stories)
            .fill(db.index())
            .ok_or_else(|| {
                anyhow!("Unable to fill out stats, an entity does not exist somewhere")
            })?;

        let query = rebuild_query(&query);

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
