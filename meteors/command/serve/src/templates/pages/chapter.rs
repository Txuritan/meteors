use crate::templates::partials::StoryCard;

pub struct Chapter<'s> {
    pub card: StoryCard<'s>,
    pub chapter: &'s str,
    pub index: usize,
}

impl<'s> Chapter<'s> {
    pub fn new(card: StoryCard<'s>, chapter: &'s str, index: usize) -> Self {
        Self {
            card,
            chapter,
            index,
        }
    }
}

include!("chapter.hbs.rs");
