use crate::templates::partials::{Pagination, StoryPartial};

#[derive(opal::Template)]
#[template(path = "pages/chapter.hbs")]
pub struct Chapter {
    pub card: StoryPartial,
    pub chapter: String,
    pub index: usize,
    pub pagination: Pagination,
}

impl Chapter {
    pub fn new(card: StoryPartial, chapter: String, index: usize) -> Self {
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
