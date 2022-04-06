#![warn(unconditional_panic, rust_2018_idioms)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use {common::prelude::*, std::env};

fn main() -> Result<()> {
    common::logger::init()?;

    let mut args: common::Args = env::args().skip(1).peekable();

    match args.next().as_deref() {
        Some("--help") => {
            println!("Usage:");
            println!("  varela");
            println!("  varela <COMMAND> [<ARGS>]");
            println!();
            println!("Options:");
            println!("  --help");
            println!();
            println!("Commands:");
            println!("  config          access and change the config");
            println!("  index           builds or updates the index");
            println!("  serve           run the internal web server [default]");
        }
        Some("config") => {
            command_config::run(args)?;
        }
        Some("index") => {
            command_index::run(args)?;
        }
        Some("serve") | None => {
            command_serve::run(args)?;
        }
        _ => {}
    }

    Ok(())
}
