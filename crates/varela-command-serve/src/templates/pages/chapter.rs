use crate::templates::partials::{Pagination, StoryPartial};

#[derive(opal::Template)]
#[template(path = "pages/chapter.hbs")]
pub struct Chapter<'s> {
    pub card: StoryPartial<'s>,
    pub chapter: &'s str,
    pub index: usize,
    pub pagination: Pagination,
}

impl<'s> Chapter<'s> {
    pub fn new(card: StoryPartial<'s>, chapter: &'s str, index: usize) -> Self {
        Self {
            chapter,
            index,
            pagination: Pagination::new(
                format!("/story/{}", card.id),
                index as u32,
                card.len as u32,
            ),
            card,
        }
    }
}
