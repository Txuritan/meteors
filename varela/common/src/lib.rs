pub mod models;

pub mod database;
pub mod logger;
pub mod utils;

pub type Args = std::iter::Peekable<std::iter::Skip<std::env::Args>>;

pub mod prelude {
    pub use {
        ::anyhow::{self, anyhow, bail, Context as _, Result},
        log::{debug, error, info, trace, warn},
        owo_colors::OwoColorize as _,
    };
}