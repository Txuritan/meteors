use crate::{search::FilledStats, templates::partials::StoryPartial};

#[derive(opal::Template)]
#[template(path = "pages/search.hbs")]
pub struct Search<'s> {
    pub stories: Vec<StoryPartial<'s>>,
    pub stats: FilledStats<'s>,
}

impl<'s> Search<'s> {
    pub fn new(stories: Vec<StoryPartial<'s>>, stats: FilledStats<'s>) -> Self {
        Self { stories, stats }
    }
}
