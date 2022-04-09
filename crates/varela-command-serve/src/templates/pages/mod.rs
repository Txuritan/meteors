pub mod chapter;
pub mod download;
pub mod index;
pub mod opds;
pub mod search;

pub use crate::templates::pages::{
    chapter::Chapter, download::Download, index::Index, search::Search,
};

#[derive(opal::Template)]
#[template(path = "pages/error.hbs")]
pub enum Error {
    Error(common::prelude::anyhow::Error),
    Http(u16, &'static str),
}

impl Error {
    pub const fn bad_request() -> Self {
        Self::Http(
            400,
            "The client sent a request that wasn't what was expected.",
        )
    }

    pub const fn internal_server_error() -> Self {
        Self::Http(503, "There was an error, check the log.")
    }

    pub const fn not_found() -> Self {
        Self::Http(404, "This page could not be found.")
    }

    pub const fn is_error(&self) -> bool {
        matches!(self, Error::Error(_))
    }

    pub const fn as_error(&self) -> Option<&common::prelude::anyhow::Error> {
        match self {
            Error::Error(err) => Some(err),
            _ => None,
        }
    }

    pub const fn as_http(&self) -> Option<(u16, &'static str)> {
        match self {
            Error::Http(code, message) => Some((*code, *message)),
            _ => None,
        }
    }
}

impl enrgy::response::IntoResponse for Error {
    fn into_response(self) -> enrgy::http::HttpResponse {
        crate::handlers::Template(self).into_response()
    }
}

impl<E: Into<common::prelude::anyhow::Error>> From<E> for Error {
    fn from(_: E) -> Self {
        todo!()
    }
}
