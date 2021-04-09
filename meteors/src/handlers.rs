use {
    crate::{
        data::{search, Database},
        prelude::*,
        router::{Context, Response},
        views::{ChapterPage, IndexPage, Layout, StoryCard},
    },
    sailfish::TemplateOnce,
    tiny_http::Header,
};

macro_rules! res {
    (200; $body:expr) => {
        Response::from_string(Res::response($body))
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
            .with_status_code(200)
    };
}

pub fn index(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or("light");

    let query = ctx.rebuild_query();

    let stories = db
        .index
        .stories
        .keys()
        .map(|id| {
            db.get_story_full(id)
                .and_then(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<StoryCard<'_>>>>()?;

    let body = Layout::new("home", theme, query, IndexPage::new(stories));

    Ok(res!(200; body))
}

pub fn story(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or("light");

    let id = ctx
        .param("id")
        .map(String::from)
        .ok_or_else(|| anyhow!("no story id was found is the request uri"))?;
    let index: usize = ctx
        .param("chapter")
        .ok_or_else(|| anyhow!("no story id was found is the request uri"))
        .and_then(|s| s.parse().map_err(anyhow::Error::from))?;

    let (_, story) = db.get_story_full(&id)?;
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

    Ok(res!(200; body))
}

pub fn search(ctx: &Context<'_, Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or("light");

    let query = ctx
        .query("search")
        .ok_or_else(|| anyhow!("search query string not found in url"))?;

    let ids = search::search(db, &query);

    let query = ctx.rebuild_query();

    let stories = ids
        .iter()
        .map(|id| {
            db.get_story_full(id)
                .and_then(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    let body = Layout::new("search", theme, query, IndexPage::new(stories));

    Ok(res!(200; body))
}

trait Res {
    fn response(self) -> String;
}

impl<'s> Res for &'s str {
    fn response(self) -> String {
        self.to_string()
    }
}

impl Res for String {
    fn response(self) -> String {
        self
    }
}

impl<I> Res for Layout<I>
where
    I: TemplateOnce,
{
    fn response(self) -> String {
        self.render_once().expect("unable to render template")
    }
}
