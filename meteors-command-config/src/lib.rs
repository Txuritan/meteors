use common::{
    database::Database,
    models::{Node, Theme},
    prelude::*,
};

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "config",
    description = "access and change the config"
)]
pub struct Command {
    #[argh(subcommand)]
    cmd: SubCommand,
}

pub fn run(command: Command) -> Result<()> {
    match command.cmd {
        SubCommand::Get(cmd) => {
            get_run(cmd)?;
        }
        SubCommand::Set(cmd) => {
            set_run(cmd)?;
        }
        SubCommand::Push(cmd) => {
            push_run(cmd)?;
        }
        SubCommand::Pop(cmd) => {
            pop_run(cmd)?;
        }
    }

    Ok(())
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum SubCommand {
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
struct GetCommand {
    #[argh(positional, description = "the key to get")]
    key: String,
}

#[inline(never)]
fn get_run(command: GetCommand) -> Result<()> {
    let database = Database::open()?;
    let settings = database.settings();

    match command.key.as_str() {
        key @ "theme" => {
            info!(target: "config", "Value of {}: {}", key.bright_blue(), settings.theme.as_class().bright_yellow());
        }
        key @ "sync-key" | key @ "sync_key" => {
            info!(target: "config", "Value of {}: {}", key.bright_blue(), settings.sync_key.bright_yellow());
        }
        key if key.starts_with("nodes.") => {
            let index = key.trim_start_matches("nodes.").parse::<usize>()?;

            if let Some(node) = settings.nodes.get(index) {
                info!(target: "config", "Value of sync node {}:", index.purple());
                info!(target: "config", "  name: {}", node.name.bright_yellow());
                info!(target: "config", "  key: {}", node.key.bright_yellow());
                info!(target: "config", "  port: {}", node.port.bright_yellow());
            } else {
                error!(target: "config", "Sync node {} does not exist", index.purple());
            }
        }
        key => {
            error!(target: "config", "Unknown config key {}", key.yellow());
        }
    }

    Ok(())
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "set", description = "set a configuration key")]
struct SetCommand {
    #[argh(positional, description = "the key to set")]
    key: String,
    #[argh(positional, description = "the value to set the key to")]
    value: String,
}

#[inline(never)]
fn set_run(command: SetCommand) -> Result<()> {
    let mut database = Database::open()?;
    let settings = database.settings_mut();

    match command.key.as_str() {
        key @ "theme" => {
            let theme = match command.value.to_lowercase().as_str() {
                "light" => Some(Theme::Light),
                "dark" => Some(Theme::Dark),
                _ => {
                    error!(target: "config", "Unknown {} value: {}", key.bright_blue(), command.value.bright_red());

                    None
                }
            };

            if let Some(theme) = theme {
                settings.theme = theme;

                info!(target: "config", "{} set to {}", key.bright_blue(), settings.theme.as_class().bright_yellow());
            }
        }
        key @ "sync-key" | key @ "sync_key" => {
            error!(target: "config", "Setting of {} is not allowed", key.bright_blue());
        }
        key if key.starts_with("nodes.") => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "push".green(), "pop".green());
        }
        key => {
            error!(target: "config", "Unknown config key {}", key.yellow());
        }
    }

    database.save()?;

    Ok(())
}

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "push",
    description = "add a entry onto a configuration list key"
)]
struct PushCommand {
    #[argh(positional, description = "the list key to add to")]
    key: String,
    #[argh(positional, description = "the value to set the key to")]
    value: String,
}

#[inline(never)]
fn push_run(command: PushCommand) -> Result<()> {
    let mut database = Database::open()?;
    let settings = database.settings_mut();

    match command.key.as_str() {
        "theme" => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "get".green(), "set".green());
        }
        "sync-key" | "sync_key" => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "get".green(), "set".green());
        }
        "nodes" => {
            let mut values = command.value.split(',');

            let name = values.next();
            let key = values.next();
            let host = values.next();
            let port = values.next();

            let zipped = name.zip(key).zip(host.zip(port));

            if let Some(((name, key), (host, port))) = zipped {
                settings.nodes.push(Node {
                    name: name.to_string(),
                    key: key.to_string(),
                    host: host.to_string(),
                    port: port.parse::<u16>()?,
                });
            } else {
                error!(target: "config", "Invalid sync node information");
            }
        }
        key => {
            error!(target: "config", "Unknown config key {}", key.yellow());
        }
    }

    Ok(())
}

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "pop",
    description = "remove an entry onto a configuration list key"
)]
pub struct PopCommand {
    #[argh(positional, description = "the list key to remove from")]
    pub key: String,
}

#[inline(never)]
fn pop_run(command: PopCommand) -> Result<()> {
    let mut database = Database::open()?;
    let settings = database.settings_mut();

    match command.key.as_str() {
        "theme" => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "get".green(), "set".green());
        }
        "sync-key" | "sync_key" => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "get".green(), "set".green());
        }
        key if key.starts_with("nodes.") => {
            let index = key.trim_start_matches("nodes.").parse::<usize>()?;

            if settings.nodes.get(index).is_some() {
                let node = settings.nodes.remove(index);

                info!(target: "config", "Removed sync node {} with value:", index.purple());
                info!(target: "config", "  name: {}", node.name.bright_yellow());
                info!(target: "config", "  key: {}", node.key.bright_yellow());
                info!(target: "config", "  port: {}", node.port.bright_yellow());
            } else {
                error!(target: "config", "Sync node {} does not exist", index.purple());
            }
        }
        key => {
            error!(target: "config", "Unknown config key {}", key.yellow());
        }
    }

    todo!()
}
