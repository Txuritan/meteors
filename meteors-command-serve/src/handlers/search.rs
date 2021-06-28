use {
    crate::{
        search,
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    qstring::QString,
    std::borrow::Cow,
    tiny_http_router::{Data, HttpResponse, Query, RawQuery},
};

fn rebuild_query(raw_query: &RawQuery) -> Cow<'static, str> {
    if raw_query.is_empty() {
        Cow::from(String::new())
    } else {
        let mut parsed = QString::from(raw_query.as_str()).into_pairs();

        parsed.retain(|(k, _)| k != "search");

        Cow::from(format!("?{}", QString::new(parsed)))
    }
}

pub fn search(
    db: Data<Database>,
    search: Query<"search">,
    query: RawQuery,
) -> Result<HttpResponse> {
    let ids = search::search(&*db, &search);

    let query = rebuild_query(&query);

    let mut stories = ids
        .iter()
        .map(|id| {
            utils::get_story_full(&*db, id)
                .and_then(|(id, story)| partials::StoryCard::new(id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    let body = Layout::new(
        Width::Slim,
        db.settings().theme,
        "search",
        query,
        pages::Index::new(stories),
    );

    Ok(crate::res!(200; body))
}

pub fn search_v2(db: Data<Database>, query: RawQuery) -> Result<HttpResponse> {
    let mut stories = db.index().stories.iter().collect::<Vec<_>>();

    let parsed_query =
        form_urlencoded::parse(query.trim_start_matches('?').as_bytes()).collect::<Vec<_>>();

    let stats = search::search_v2(&parsed_query[..], &mut stories)
        .fill(db.index())
        .ok_or_else(|| anyhow!("Unable to fill out stats, an entity does not exist somewhere"))?;

    let query = rebuild_query(&query);

    let mut stories = stories
        .into_iter()
        .map(|(id, _)| {
            utils::get_story_full(&*db, id)
                .and_then(|(id, story)| partials::StoryCard::new(id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    let body = Layout::new(
        Width::Wide,
        db.settings().theme,
        "search",
        query,
        pages::Search::new(stories, stats),
    );

    Ok(crate::res!(200; body))
}
