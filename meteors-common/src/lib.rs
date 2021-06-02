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

pub fn action<T>(name: &'static str, ctx: &T, run: fn(&T) -> anyhow::Result<()>) {
    use owo_colors::OwoColorize as _;

    // TODO: make this a indented log
    if let Err(err) = run(ctx) {
        log::error!("{} unable to run command `{}`", "+".bright_black(), name);

        for cause in err.chain() {
            log::error!("{} {:?}", "+".bright_black(), cause);
        }
    }
}

pub trait Action {
    fn run(&self) -> anyhow::Result<()>;
}
