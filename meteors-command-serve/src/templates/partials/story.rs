use {
    crate::templates::{
        partials::{OriginList, TagList},
        TagKind,
    },
    common::{
        models::{resolved, Entity, Rating, StoryInfo},
        prelude::*,
    },
    std::borrow::Cow,
};

pub struct StoryCard<'s> {
    pub id: &'s str,

    pub len: usize,
    pub info: StoryInfo,

    pub rating: Rating,
    pub categories: Vec<Entity>,
    pub authors: Vec<Entity>,

    pub origins: OriginList,
    pub tags: TagList,

    pub query: Cow<'static, str>,
}

impl<'s> StoryCard<'s> {
    pub fn new(id: &'s str, story: resolved::Story, query: Cow<'static, str>) -> Result<Self> {
        let resolved::StoryMeta {
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

            len: story.chapters.len(),
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

    pub fn title(&self) -> &str {
        &self.info.title
    }
}

include!("story.hbs.rs");
