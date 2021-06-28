use {crate::templates::Width, common::models::Theme, opal::Template, std::borrow::Cow};

pub struct Layout<B>
where
    B: Template,
{
    width: Width,
    theme: Theme,
    title: String,
    query: Option<Cow<'static, str>>,
    body: B,
}

impl<B> Layout<B>
where
    B: Template,
{
    #[allow(clippy::needless_pass_by_value)]
    pub fn new<S>(width: Width, theme: Theme, title: S, query: Option<Cow<'static, str>>, body: B) -> Self
    where
        S: ToString,
    {
        Self {
            width,
            theme,
            title: title.to_string(),
            query,
            body,
        }
    }
}

include!("layout.hbs.rs");
