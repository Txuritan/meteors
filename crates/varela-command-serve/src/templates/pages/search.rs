use crate::{search::FilledStats, templates::partials::StoryPartial};

#[derive(opal::Template)]
#[template(path = "pages/search.hbs")]
pub struct Search {
    pub stories: Vec<StoryPartial>,
    pub stats: FilledStats,
}

impl Search {
    pub fn new(stories: Vec<StoryPartial>, stats: FilledStats) -> Self {
        Self { stories, stats }
    }
}
