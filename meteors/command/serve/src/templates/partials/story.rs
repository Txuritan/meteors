use {
    crate::templates::{
        partials::{Contrast, Link, OriginList, TagList},
        TagKind,
    },
    common::{
        models::{Entity, Existing, Rating, ResolvedStory, ResolvedStoryMeta, StoryInfo},
        prelude::*,
    },
    std::borrow::Cow,
};

#[derive(opal::Template)]
#[template(path = "partials/story.hbs")]
pub struct StoryPartial<'s> {
    pub id: &'s str,

    pub len: usize,
    pub info: StoryInfo,

    pub rating: Rating,
    pub categories: Vec<Existing<Entity>>,
    pub authors: Vec<Existing<Entity>>,

    pub origins: OriginList,
    pub tags: TagList,

    pub query: Option<Cow<'static, str>>,
}

impl<'s> StoryPartial<'s> {
    pub fn new(
        id: &'s str,
        story: ResolvedStory,
        query: Option<Cow<'static, str>>,
    ) -> Result<Self> {
        let ResolvedStoryMeta {
            rating,
            authors,
            categories,
            origins,
            warnings,
            pairings,
            characters,
            generals,
        } = story.meta;

        Ok(StoryPartial {
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

    fn push(
        tags: &mut Vec<(TagKind, Existing<Entity>)>,
        kind: TagKind,
        list: Vec<Existing<Entity>>,
    ) {
        for entity in list {
            tags.push((kind, entity));
        }
    }

    pub fn title(&self) -> &str {
        &self.info.title
    }
}
