use {
    crate::{
        data::{search, Database},
        models::{
            proto::{Entity, Rating, StoryInfo},
            StoryFull,
        },
        prelude::*,
        router::{Context, Response},
    },
    sailfish::TemplateOnce,
    std::borrow::Cow,
    tiny_http::Header,
};

static CSS: &str = include_str!("../assets/style.css");

macro_rules! res {
    (200; $body:expr) => {
        Response::from_string(Res::response($body))
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
            .with_status_code(200)
    };
}

pub fn index(ctx: &Context<Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or("light");

    let query = ctx.rebuild_query();

    let stories = db
        .index
        .stories
        .keys()
        .map(|id| {
            db.get_story_full(id)
                .map(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<StoryCard>>>()?;

    let body = Layout::new("home", theme, query, IndexPage { stories });

    Ok(res!(200; body))
}

pub fn story(ctx: &Context<Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or("light");

    let id = ctx
        .param("id")
        .map(String::from)
        .ok_or_else(|| anyhow!("no story id was found is the request uri"))?;
    let chapter: usize = ctx
        .param("chapter")
        .ok_or_else(|| anyhow!("no story id was found is the request uri"))
        .and_then(|s| s.parse().map_err(anyhow::Error::from))?;

    let (_, story) = db.get_story_full(&id)?;
    let story_body = db.get_chapter_body(&id, chapter)?;

    let query = ctx.rebuild_query();

    let body = Layout::new(
        story.info.title.clone(),
        theme,
        query.clone(),
        ChapterPage {
            card: StoryCard::new(&id, story, query.clone()),
            chapter: &story_body,
            index: chapter,
            query,
        },
    );

    Ok(res!(200; body))
}

pub fn search(ctx: &Context<Database>) -> Result<Response> {
    let db = ctx.state();

    let theme = ctx.query("theme").unwrap_or("light");

    let query = ctx
        .query("search")
        .ok_or_else(|| anyhow!("search query string not found in url"))?;

    let ids = search::search(db, query);

    let query = ctx.rebuild_query();

    let stories = ids
        .iter()
        .map(|id| {
            db.get_story_full(id)
                .map(|(id, story)| StoryCard::new(&id, story, query.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    let body = Layout::new("search", theme, query, IndexPage { stories });

    Ok(res!(200; body))
}

#[derive(TemplateOnce)]
#[template(path = "pages/index.stpl")]
struct IndexPage<'s> {
    stories: Vec<StoryCard<'s>>,
}

#[derive(TemplateOnce)]
#[template(path = "pages/chapter.stpl")]
struct ChapterPage<'s> {
    card: StoryCard<'s>,
    chapter: &'s str,
    index: usize,

    query: Cow<'static, str>,
}

#[derive(TemplateOnce)]
#[template(path = "layout.stpl")]
struct Layout<B>
where
    B: TemplateOnce,
{
    css: &'static str,
    title: String,
    theme: String,
    query: Cow<'static, str>,
    body: B,
}

impl<B> Layout<B>
where
    B: TemplateOnce,
{
    #[allow(clippy::needless_pass_by_value)]
    fn new<S, T>(title: S, theme: T, query: Cow<'static, str>, body: B) -> Self
    where
        S: ToString,
        T: ToString,
    {
        Self {
            css: CSS,
            title: title.to_string(),
            theme: theme.to_string(),
            query,
            body,
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "partials/story.stpl")]
struct StoryCard<'s> {
    id: &'s str,

    file_name: String,
    length: usize,
    chapters: usize,

    info: StoryInfo,

    rating: Rating,

    authors: Vec<Entity>,

    warnings: TagList<'s>,
    pairings: TagList<'s>,
    characters: TagList<'s>,
    generals: TagList<'s>,

    query: Cow<'static, str>,
}

impl<'s> StoryCard<'s> {
    pub fn new(id: &'s str, story: StoryFull, query: Cow<'static, str>) -> Self {
        StoryCard {
            id,

            file_name: story.file_name,
            length: story.length as usize,
            chapters: story.chapters.len(),

            info: story.info,

            rating: story.meta.rating,

            authors: story.meta.authors,

            warnings: TagList {
                kind: "warnings",
                tags: story.meta.warnings,
            },
            pairings: TagList {
                kind: "pairings",
                tags: story.meta.pairings,
            },
            characters: TagList {
                kind: "characters",
                tags: story.meta.characters,
            },
            generals: TagList {
                kind: "generals",
                tags: story.meta.generals,
            },

            query,
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "partials/tag-list.stpl")]
struct TagList<'s> {
    kind: &'s str,
    tags: Vec<Entity>,
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
        self.render_once().unwrap()
    }
}
