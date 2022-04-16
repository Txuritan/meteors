use common::models::Theme;
use opal::Template;

use crate::templates::{pages, partials::nav, Width};

#[derive(opal::Template)]
#[template(path = "layout.hbs")]
pub struct Layout<B>
where
    B: Template,
{
    width: Width,
    theme: Theme,
    title: String,
    nav: nav::Nav,
    query: Option<String>,
    body: B,
}

impl<B> Layout<B>
where
    B: Template,
{
    #[allow(clippy::needless_pass_by_value)]
    pub fn new<S>(width: Width, theme: Theme, title: S, query: Option<String>, body: B) -> Self
    where
        S: ToString,
    {
        Self {
            width,
            theme,
            title: title.to_string(),
            query,
            nav: nav::NAV,
            body,
        }
    }
}

impl Layout<pages::Error> {
    fn error(title: String, body: pages::Error) -> Self {
        Self {
            width: Width::Slim,
            theme: Theme::Dark,
            title,
            nav: nav::NAV,
            query: None,
            body,
        }
    }

    pub fn internal_server_error() -> Self {
        Self::error("503".to_string(), pages::Error::internal_server_error())
    }

    pub fn not_found() -> Self {
        Self::error("404".to_string(), pages::Error::not_found())
    }
}
