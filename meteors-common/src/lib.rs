pub mod models;

pub mod database;
pub mod logger;
pub mod utils;

pub mod prelude {
    pub use {
        crate::utils::new_id,
        ::anyhow::{self, anyhow, bail, Context as _, Result},
        either::{Left, Right},
        log::{debug, error, info, trace, warn},
        owo_colors::OwoColorize as _,
    };
}

pub use prost::Message;
