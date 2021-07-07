use crate::templates::partials::StoryCard;

#[derive(opal::Template)]
#[template(path = "pages/chapter.hbs")]
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
