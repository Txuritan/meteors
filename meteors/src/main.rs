#![warn(unconditional_panic, rust_2018_idioms)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use common::{prelude::*, Action as _};

#[derive(argh::FromArgs)]
#[argh(description = "an offline archive of our own viewer")]
struct Args {
    #[argh(subcommand)]
    cmd: SubCommand,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Config(command_config::Command),
    Index(command_index::Command),
    Serve(command_serve::Command),
}

fn main() -> Result<()> {
    common::logger::init()?;

    let args: Args = argh::from_env();

    match args.cmd {
        SubCommand::Config(cmd) => {
            cmd.run()?;
        }
        SubCommand::Index(cmd) => {
            cmd.run()?;
        }
        SubCommand::Serve(cmd) => {
            cmd.run()?;
        }
    }

    Ok(())
}
