use crate::{search::FilledStats, templates::partials::StoryCard};

#[derive(opal::Template)]
#[template(path = "pages/search.hbs")]
pub struct Search<'s> {
    pub stories: Vec<StoryCard<'s>>,
    pub stats: FilledStats<'s>,
}

impl<'s> Search<'s> {
    pub fn new(stories: Vec<StoryCard<'s>>, stats: FilledStats<'s>) -> Self {
        Self { stories, stats }
    }
}
