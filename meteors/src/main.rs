#![warn(unconditional_panic, rust_2018_idioms)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use {common::prelude::*, seahorse::App, std::env};

fn main() -> Result<()> {
    common::logger::init()?;

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .command(command_serve::command());

    app.run(env::args().collect());

    Ok(())
}
