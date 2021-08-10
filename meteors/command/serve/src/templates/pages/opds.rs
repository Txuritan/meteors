use common::models::{Existing, ResolvedStory};

#[derive(opal::Template)]
#[template(path = "pages/opds-feed.hbs")]
pub struct OpdsFeed {
    pub updated: String,
    pub stories: Vec<Existing<ResolvedStory>>,
}

impl OpdsFeed {
    pub fn new(updated: String, stories: Vec<Existing<ResolvedStory>>) -> Self {
        Self { updated, stories }
    }
}
