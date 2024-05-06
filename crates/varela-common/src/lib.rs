pub mod database;
pub mod logger;
pub mod models;
pub mod utils;

pub type Args = std::iter::Peekable<std::iter::Skip<std::env::Args>>;

pub static ICON: &[u8] = include_bytes!("../assets/noel.ico");

pub mod prelude {
    pub use {
        ::anyhow::{self, anyhow, bail, Context as _, Result},
        vfmt::{debug, error, info, trace, utils::colorize::Colorize as _, warn},
    };
}
