use crate::templates::partials::StoryPartial;

#[derive(opal::Template)]
#[template(path = "pages/index.hbs")]
pub struct Index<'s> {
    pub stories: Vec<StoryPartial<'s>>,
}

impl<'s> Index<'s> {
    pub fn new(stories: Vec<StoryPartial<'s>>) -> Self {
        Self { stories }
    }
}
