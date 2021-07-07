use crate::templates::partials::StoryCard;

#[derive(opal::Template)]
#[template(path = "pages/index.hbs")]
pub struct Index<'s> {
    pub stories: Vec<StoryCard<'s>>,
}

impl<'s> Index<'s> {
    pub fn new(stories: Vec<StoryCard<'s>>) -> Self {
        Self { stories }
    }
}
