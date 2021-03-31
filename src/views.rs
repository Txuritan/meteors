use {
    crate::{
        models::{
            proto::{Entity, Rating, StoryInfo},
            StoryFull, StoryFullMeta,
        },
        prelude::*,
    },
    sailfish::TemplateOnce,
    std::borrow::Cow,
};

static CSS: &str = include_str!("../assets/style.css");

#[derive(TemplateOnce)]
#[template(path = "pages/index.stpl")]
pub struct IndexPage<'s> {
    pub stories: Vec<StoryCard<'s>>,
}

#[derive(TemplateOnce)]
#[template(path = "pages/chapter.stpl")]
pub struct ChapterPage<'s> {
    pub card: StoryCard<'s>,
    pub chapter: &'s str,
    pub index: usize,

    pub query: Cow<'static, str>,
}

#[derive(TemplateOnce)]
#[template(path = "layout.stpl")]
pub struct Layout<B>
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
    pub fn new<S, T>(title: S, theme: T, query: Cow<'static, str>, body: B) -> Self
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

#[derive(Debug, Clone, Copy)]
pub enum TagKind {
    Warning,
    Pairing,
    Character,
    General,
}

impl TagKind {
    pub const fn class(self) -> &'static str {
        match self {
            TagKind::Warning => "warning",
            TagKind::Pairing => "pairing",
            TagKind::Character => "character",
            TagKind::General => "general",
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "partials/story.stpl")]
pub struct StoryCard<'s> {
    id: &'s str,

    chapters: usize,
    info: StoryInfo,

    rating: Rating,
    categories: Vec<Entity>,
    authors: Vec<Entity>,

    origins: OriginList,
    tags: TagList,

    query: Cow<'static, str>,
}

impl<'s> StoryCard<'s> {
    pub fn new(id: &'s str, story: StoryFull, query: Cow<'static, str>) -> Result<Self> {
        let StoryFullMeta {
            rating,
            authors,
            categories,
            origins,
            warnings,
            pairings,
            characters,
            generals,
        } = story.meta;

        Ok(StoryCard {
            id,

            chapters: story.chapters.len(),
            info: story.info,

            rating,
            categories,
            authors,

            origins: OriginList { origins },
            tags: TagList {
                tags: {
                    let mut tags = Vec::with_capacity(
                        warnings.len() + pairings.len() + characters.len() + generals.len(),
                    );

                    Self::push(&mut tags, TagKind::Warning, warnings);
                    Self::push(&mut tags, TagKind::Pairing, pairings);
                    Self::push(&mut tags, TagKind::Character, characters);
                    Self::push(&mut tags, TagKind::General, generals);

                    tags
                },
            },

            query,
        })
    }

    fn push(tags: &mut Vec<(TagKind, Entity)>, kind: TagKind, list: Vec<Entity>) {
        for entity in list {
            tags.push((kind, entity));
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "partials/origin-list.stpl")]
pub struct OriginList {
    pub origins: Vec<Entity>,
}

#[derive(TemplateOnce)]
#[template(path = "partials/tag-list.stpl")]
pub struct TagList {
    pub tags: Vec<(TagKind, Entity)>,
}
