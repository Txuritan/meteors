use crate::templates::partials::StoryCard;

pub struct Index<'s> {
    pub stories: Vec<StoryCard<'s>>,
}

impl<'s> Index<'s> {
    pub fn new(stories: Vec<StoryCard<'s>>) -> Self {
        Self { stories }
    }
}

include!("index.hbs.rs");
