use crate::templates::partials::StoryPartial;

#[derive(opal::Template)]
#[template(path = "pages/index.hbs")]
pub struct Index {
    pub stories: Vec<StoryPartial>,
}

impl Index {
    pub fn new(stories: Vec<StoryPartial>) -> Self {
        Self { stories }
    }
}
