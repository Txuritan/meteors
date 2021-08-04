use crate::templates::partials::StoryPartial;

#[derive(opal::Template)]
#[template(path = "pages/chapter.hbs")]
pub struct Chapter<'s> {
    pub card: StoryPartial<'s>,
    pub chapter: &'s str,
    pub index: usize,
}

impl<'s> Chapter<'s> {
    pub fn new(card: StoryPartial<'s>, chapter: &'s str, index: usize) -> Self {
        Self {
            card,
            chapter,
            index,
        }
    }
}
