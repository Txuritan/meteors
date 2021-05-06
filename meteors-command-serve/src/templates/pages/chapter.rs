use {crate::templates::partials::StoryCard, std::borrow::Cow};

pub struct Chapter<'s> {
    pub card: StoryCard<'s>,
    pub chapter: &'s str,
    pub index: usize,

    pub query: Cow<'static, str>,
}

impl<'s> Chapter<'s> {
    pub fn new(
        card: StoryCard<'s>,
        chapter: &'s str,
        index: usize,
        query: Cow<'static, str>,
    ) -> Self {
        Self {
            card,
            chapter,
            index,
            query,
        }
    }
}

include!("chapter.hbs.rs");
