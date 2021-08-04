use common::{
    database::Database,
    models::{Node, Theme},
    prelude::*,
};

pub fn run(mut args: common::Args) -> Result<()> {
    match args.next().as_deref() {
        Some("--help") => {
            println!("Usage:");
            println!("  meteors config <COMMAND> [<ARGS>]");
            println!();
            println!("Options:");
            println!("  --help");
            println!();
            println!("Commands:");
            println!("  get             get and prints a configuration key");
            println!("  set             set a configuration key");
            println!("  push            add a entry onto a configuration list key");
            println!("  pop             remove an entry onto a configuration list key");
        }
        Some("get") => {
            run_get(args)?;
        }
        Some("set") => {
            run_set(args)?;
        }
        Some("push") => {
            run_push(args)?;
        }
        Some("pop") => {
            run_pop(args)?;
        }
        _ => {}
    }

    Ok(())
}

#[inline(never)]
fn run_get(mut args: common::Args) -> Result<()> {
    let database = Database::open()?;
    let settings = database.settings();

    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        println!("Usage:");
        println!("  meteors config get <KEY>");
        println!();
        println!("Options:");
        println!("  --help");
        println!();
        println!("Arguments:");
        println!("  key             the key to get");

        return Ok(());
    }

    let key = args.next().ok_or_else(|| anyhow::anyhow!("`config get` is missing `key` value"))?;

    match key.as_str() {
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

#[inline(never)]
fn run_set(mut args: common::Args) -> Result<()> {
    let mut database = Database::open()?;
    let settings = database.settings_mut();

    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        println!("Usage:");
        println!("  meteors config set <KEY> <VALUE>");
        println!();
        println!("Options:");
        println!("  --help");
        println!();
        println!("Arguments:");
        println!("  key             the key to set");
        println!("  value           the value to set the key to");

        return Ok(());
    }

    let key = args.next().ok_or_else(|| anyhow::anyhow!("`config set` is missing `key` value"))?;
    let value = args.next().ok_or_else(|| anyhow::anyhow!("`config set` is missing `value` value"))?;

    match key.as_str() {
        key @ "theme" => {
            let theme = match value.to_lowercase().as_str() {
                "light" => Some(Theme::Light),
                "dark" => Some(Theme::Dark),
                _ => {
                    error!(target: "config", "Unknown {} value: {}", key.bright_blue(), value.bright_red());

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

#[inline(never)]
fn run_push(mut args: common::Args) -> Result<()> {
    let mut database = Database::open()?;
    let settings = database.settings_mut();

    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        println!("Usage:");
        println!("  meteors config push <KEY> <VALUE>");
        println!();
        println!("Options:");
        println!("  --help");
        println!();
        println!("Arguments:");
        println!("  key             the list to add to");
        println!("  value           the value to add to the list");

        return Ok(());
    }

    let key = args.next().ok_or_else(|| anyhow::anyhow!("`config push` is missing `key` value"))?;
    let value = args.next().ok_or_else(|| anyhow::anyhow!("`config push` is missing `value` value"))?;

    match key.as_str() {
        "theme" => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "get".green(), "set".green());
        }
        "sync-key" | "sync_key" => {
            error!(target: "config", "Setting of {} is not allowed, use the {} or {} command", "nodes".bright_blue(), "get".green(), "set".green());
        }
        "nodes" => {
            let mut values = value.split(',');

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

#[inline(never)]
fn run_pop(mut args: common::Args) -> Result<()> {
    let mut database = Database::open()?;
    let settings = database.settings_mut();

    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        println!("Usage:");
        println!("  meteors config pop <KEY>");
        println!();
        println!("Options:");
        println!("  --help");
        println!();
        println!("Arguments:");
        println!("  key             the list item to remove");

        return Ok(());
    }

    let key = args.next().ok_or_else(|| anyhow::anyhow!("`config pop` is missing `key` value"))?;

    match key.as_str() {
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
