use common::{
    database::Database,
    models::{Node, Theme},
    prelude::*,
    Action,
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

impl Action for Command {
    fn run(&self) -> Result<()> {
        match &self.cmd {
            SubCommand::Get(cmd) => {
                cmd.run()?;
            }
            SubCommand::Set(cmd) => {
                cmd.run()?;
            }
            SubCommand::Push(cmd) => {
                cmd.run()?;
            }
            SubCommand::Pop(cmd) => {
                cmd.run()?;
            }
        }

        Ok(())
    }
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

impl Action for GetCommand {
    fn run(&self) -> Result<()> {
        let database = Database::open()?;
        let settings = database.settings();

        match self.key.as_str() {
            key @ "theme" => {
                info!(target: "config", "{} Value of {}: {}", "+".bright_black(), key.bright_blue(), settings.theme.as_class().bright_yellow());
            }
            key @ "sync-key" | key @ "sync_key" => {
                info!(target: "config", "{} Value of {}: {}", "+".bright_black(), key.bright_blue(), settings.sync_key.bright_yellow());
            }
            key if key.starts_with("nodes.") => {
                let index = key.trim_start_matches("nodes.").parse::<usize>()?;

                if let Some(node) = settings.nodes.get(index) {
                    info!(target: "config", "{} {} Value of sync node {}:", "+".bright_black(), "+".bright_black(), index.purple());
                    info!(target: "config", "  {} name: {}", "+".bright_black(), node.name.bright_yellow());
                    info!(target: "config", "  {} key: {}", "+".bright_black(), node.key.bright_yellow());
                    info!(target: "config", "  {} port: {}", "+".bright_black(), node.port.bright_yellow());
                } else {
                    error!(target: "config", "{} {} Sync node {} does not exist", "+".bright_black(), "+".bright_black(), index.purple());
                }
            }
            key => {
                error!(target: "config", "{} Unknown config key {}", "+".bright_black(), key.yellow());
            }
        }

        Ok(())
    }
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "set", description = "set a configuration key")]
struct SetCommand {
    #[argh(positional, description = "the key to set")]
    key: String,
    #[argh(positional, description = "the value to set the key to")]
    value: String,
}

impl Action for SetCommand {
    fn run(&self) -> Result<()> {
        let mut database = Database::open()?;
        let settings = database.settings_mut();

        match self.key.as_str() {
            key @ "theme" => {
                let theme = match self.value.to_lowercase().as_str() {
                    "light" => Some(Theme::Light),
                    "dark" => Some(Theme::Dark),
                    _ => {
                        error!(target: "config", "{} Unknown {} value: {}", "+".bright_black(), key.bright_blue(), self.value.bright_red());

                        None
                    }
                };

                if let Some(theme) = theme {
                    settings.theme = theme;

                    info!(target: "config", "{} {} set to {}", "+".bright_black(), key.bright_blue(), settings.theme.as_class().bright_yellow());
                }
            }
            key @ "sync-key" | key @ "sync_key" => {
                error!(target: "config", "{} Setting of {} is not allowed", "+".bright_black(), key.bright_blue());
            }
            key if key.starts_with("nodes.") => {
                error!(target: "config", "{} Setting of {} is not allowed, use the {} or {} command", "+".bright_black(), "nodes".bright_blue(), "push".green(), "pop".green());
            }
            key => {
                error!(target: "config", "{} Unknown config key {}", "+".bright_black(), key.yellow());
            }
        }

        database.save()?;

        Ok(())
    }
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

impl Action for PushCommand {
    fn run(&self) -> Result<()> {
        let mut database = Database::open()?;
        let settings = database.settings_mut();

        match self.key.as_str() {
            "theme" => {
                error!(target: "config", "{} Setting of {} is not allowed, use the {} or {} command", "+".bright_black(), "nodes".bright_blue(), "get".green(), "set".green());
            }
            "sync-key" | "sync_key" => {
                error!(target: "config", "{} Setting of {} is not allowed, use the {} or {} command", "+".bright_black(), "nodes".bright_blue(), "get".green(), "set".green());
            }
            "nodes" => {
                let mut values = self.value.split(',');

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
                    error!(target: "config", "{} Invalid sync node information", "+".bright_black());
                }
            }
            key => {
                error!(target: "config", "{} Unknown config key {}", "+".bright_black(), key.yellow());
            }
        }

        Ok(())
    }
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

impl Action for PopCommand {
    fn run(&self) -> Result<()> {
        let mut database = Database::open()?;
        let settings = database.settings_mut();

        match self.key.as_str() {
            "theme" => {
                error!(target: "config", "{} Setting of {} is not allowed, use the {} or {} command", "+".bright_black(), "nodes".bright_blue(), "get".green(), "set".green());
            }
            "sync-key" | "sync_key" => {
                error!(target: "config", "{} Setting of {} is not allowed, use the {} or {} command", "+".bright_black(), "nodes".bright_blue(), "get".green(), "set".green());
            }
            key if key.starts_with("nodes.") => {
                let index = key.trim_start_matches("nodes.").parse::<usize>()?;

                if settings.nodes.get(index).is_some() {
                    let node = settings.nodes.remove(index);

                    info!(target: "config", "{} {} Removed sync node {} with value:", "+".bright_black(), "+".bright_black(), index.purple());
                    info!(target: "config", "  {} name: {}", "+".bright_black(), node.name.bright_yellow());
                    info!(target: "config", "  {} key: {}", "+".bright_black(), node.key.bright_yellow());
                    info!(target: "config", "  {} port: {}", "+".bright_black(), node.port.bright_yellow());
                } else {
                    error!(target: "config", "{} {} Sync node {} does not exist", "+".bright_black(), "+".bright_black(), index.purple());
                }
            }
            key => {
                error!(target: "config", "{} Unknown config key {}", "+".bright_black(), key.yellow());
            }
        }

        todo!()
    }
}
