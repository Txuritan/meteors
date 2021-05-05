use common::{prelude::*, Action};

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "config",
    description = "access and change the config"
)]
pub struct Command {
    #[argh(subcommand)]
    pub cmd: SubCommand,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
pub enum SubCommand {
    Get(GetCommand),
    Set(SetCommand),
    Push(PushCommand),
    Pop(PopCommand),
}

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "get",
    description = "get and prints a configuration key"
)]
pub struct GetCommand {
    #[argh(option, description = "the key to get")]
    pub key: String,
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "set", description = "set a configuration key")]
pub struct SetCommand {
    #[argh(option, description = "the key to set")]
    pub key: String,
    #[argh(option, description = "the value to set the key to")]
    pub value: String,
}

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "push",
    description = "add a entry onto a configuration list key"
)]
pub struct PushCommand {
    #[argh(option, description = "the list key to add to")]
    pub key: String,
    #[argh(option, description = "the value to set the key to")]
    pub value: String,
}

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "pop",
    description = "remove an entry onto a configuration list key"
)]
pub struct PopCommand {
    #[argh(option, description = "the list key to remove from")]
    pub key: String,
}

impl Action for Command {
    fn run(&self) -> Result<()> {
        // let action = ctx.string_flag("action")?;
        // let key = ctx.string_flag("key")?;

        // match action.as_str() {
        //     "get" => {}
        //     "set" => {}
        //     "push" => {}
        //     _ => {}
        // }

        Ok(())
    }
}
