pub mod colorize;
pub mod database;
pub mod logger;
pub mod models;
pub mod utils;

pub type Args = std::iter::Peekable<std::iter::Skip<std::env::Args>>;

pub static ICON: &[u8] = include_bytes!("../assets/noel.ico");

pub mod prelude {
    pub use {
        crate::colorize::Colorize as _,
        ::anyhow::{self, anyhow, bail, Context as _, Result},
        vfmt_log::{debug, error, info, trace, warn},
    };
}
