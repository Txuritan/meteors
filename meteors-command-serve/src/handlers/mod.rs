mod index;
mod search;
mod story;

pub use crate::handlers::{index::index, search::search, story::story};

use {crate::views::Layout, sailfish::TemplateOnce};

#[macro_export]
macro_rules! res {
    (200; $body:expr) => {
        $crate::router::Response::from_string($crate::handlers::Res::response($body))
            .with_header(
                ::tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                )
                .unwrap(),
            )
            .with_status_code(200)
    };
}

pub trait Res {
    fn response(self) -> String;
}

impl<'s> Res for &'s str {
    fn response(self) -> String {
        self.to_string()
    }
}

impl Res for String {
    fn response(self) -> String {
        self
    }
}

impl<I> Res for Layout<I>
where
    I: TemplateOnce,
{
    fn response(self) -> String {
        self.render_once().expect("unable to render template")
    }
}
