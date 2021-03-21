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

pub fn index(ctx: Context<Database>) -> Result<Response> {
    let db = ctx.state();

    let stories = db
        .index
        .stories
        .keys()
        .map(|id| {
            db.get_story_full(id)
                .map(|(id, story)| StoryCard::new(&id, story))
        })
        .collect::<Result<Vec<StoryCard>>>()?;

    let body = Layout::new("home", IndexPage { stories });

    Ok(res!(200; body))
}

pub fn story(ctx: Context<Database>) -> Result<Response> {
    let db = ctx.state();

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

    let body = Layout::new(
        story.info.title.clone(),
        ChapterPage {
            card: StoryCard::new(&id, story),
            chapter: &story_body,
            index: chapter,
        },
    );

    Ok(res!(200; body))
}

pub fn search(ctx: Context<Database>) -> Result<Response> {
    let db = ctx.state();

    // path?query ? ( query = key=value[&key=value] ) => Vec{(key, value) [, (key, value)]}
    let queries = ctx
        .query()
        .map(|query| {
            query
                .split('&')
                .map(|param| {
                    param
                        .contains('=')
                        .then(|| {
                            param.find('=').map(|index| {
                                let (key, value) = param.split_at(index);

                                (key, Some(value))
                            })
                        })
                        .flatten()
                        .unwrap_or((param, None))
                })
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| anyhow!("no query parameters found in url"))?;

    let query = queries
        .into_iter()
        .find(|(key, _)| *key == "search")
        .and_then(|(_, value)| value)
        .ok_or_else(|| anyhow!("search query string not found in url"))?;

    let ids = search::search(db, query);

    let stories = ids
        .iter()
        .map(|id| {
            db.get_story_full(id)
                .map(|(id, story)| StoryCard::new(&id, story))
        })
        .collect::<Result<Vec<_>>>()?;

    let body = Layout::new("search", IndexPage { stories });

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
}

#[derive(TemplateOnce)]
#[template(path = "layout.stpl")]
struct Layout<B>
where
    B: TemplateOnce,
{
    css: &'static str,
    title: String,
    body: B,
}

impl<B> Layout<B>
where
    B: TemplateOnce,
{
    fn new<S>(title: S, body: B) -> Self
    where
        S: ToString,
    {
        Self {
            css: CSS,
            title: title.to_string(),
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
}

impl<'s> StoryCard<'s> {
    pub fn new(id: &'s str, story: StoryFull) -> Self {
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
