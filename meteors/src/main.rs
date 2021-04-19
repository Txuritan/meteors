#![warn(unconditional_panic, rust_2018_idioms)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod commands;
mod data;
mod format;
mod models;
mod regex;

mod gztar;
mod logger;
mod utils;

mod prelude {
    pub use {
        crate::utils::new_id,
        ::anyhow::{self, anyhow, bail, Context as _, Result},
        either::{Left, Right},
        log::{debug, error, info, trace, warn},
        owo_colors::OwoColorize as _,
    };
}

use {
    crate::{commands::serve, prelude::*},
    seahorse::App,
    std::env,
};

fn main() -> Result<()> {
    logger::init()?;

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .command(serve::command());

    app.run(env::args().collect());

    Ok(())
}
