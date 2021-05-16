pub mod chapter;
pub mod download;
pub mod index;
pub mod search;

pub use crate::templates::pages::{
    chapter::Chapter, download::Download, index::Index, search::Search,
};
