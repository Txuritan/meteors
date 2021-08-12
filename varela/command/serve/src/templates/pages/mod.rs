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
pub struct Error {
    error: &'static str,
    message: &'static str,
}

impl Error {
    pub const fn internal_server_error() -> Self {
        Self { error: "503", message: "There was an error, check the log." }
    }

    pub const fn not_found() -> Self {
        Self { error: "404", message: "This page could not be found." }
    }
}
