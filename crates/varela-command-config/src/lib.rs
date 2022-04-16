use std::path::PathBuf;

use common::{
    database::Database,
    models::{Node, Theme},
    prelude::*,
};

pub fn run(mut args: common::Args) -> Result<()> {
    match args.next().as_deref() {
        Some("--help") => {
            vfmt::println!("Usage:");
            vfmt::println!("  varela config <COMMAND> [<ARGS>]");
            vfmt::println!();
            vfmt::println!("Options:");
            vfmt::println!("  --help");
            vfmt::println!();
            vfmt::println!("Commands:");
            vfmt::println!("  get             get and prints a configuration key");
            vfmt::println!("  set             set a configuration key");
            vfmt::println!("  push            add a entry onto a configuration list key");
            vfmt::println!("  pop             remove an entry onto a configuration list key");
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
        vfmt::println!("Usage:");
        vfmt::println!("  varela config get <KEY>");
        vfmt::println!();
        vfmt::println!("Options:");
        vfmt::println!("  --help");
        vfmt::println!();
        vfmt::println!("Arguments:");
        vfmt::println!("  key             the key to get");

        return Ok(());
    }

    let key = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("`config get` is missing `key` value"))?;

    match key.as_str() {
        "theme" => {
            info!(target: "config", "Value of {}: {}", "theme".bright_blue(), settings.theme.as_class().bright_yellow());
        }
        "sync-key" | "sync_key" => {
            info!(target: "config", "Value of {}: {}", "sync-key" .bright_blue(), settings.sync_key.bright_yellow());
        }
        "data_path" | "data-path" => {
            info!(target: "config", "Value of {}: {}", "data-path".bright_blue(), settings.data_path.bright_yellow());
        }
        "temp_path" | "temp-path" => {
            info!(target: "config", "Value of {}: {}", "temp-path".bright_blue(), settings.temp_path.bright_yellow());
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

    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        vfmt::println!("Usage:");
        vfmt::println!("  varela config set <KEY> <VALUE>");
        vfmt::println!();
        vfmt::println!("Options:");
        vfmt::println!("  --help");
        vfmt::println!();
        vfmt::println!("Arguments:");
        vfmt::println!("  key             the key to set");
        vfmt::println!("  value           the value to set the key to");

        return Ok(());
    }

    let key = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("`config set` is missing `key` value"))?;
    let value = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("`config set` is missing `value` value"))?;

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
                database.settings_mut().theme = theme;

                info!(target: "config", "{} set to {}", key.bright_blue(), database.settings().theme.as_class().bright_yellow());
            }
        }
        "sync-key" | "sync_key" => {
            error!(target: "config", "Setting of {} is not allowed", "sync-key".bright_blue());
        }
        "data-path" | "data_path" => {
            let path = PathBuf::from(&value);

            if path.exists() {
                database.data_path = path;
                database.settings_mut().data_path = value;
            } else {
                error!(target: "config", "Setting of {} is not committed, supplied path does not exist", "data-path".bright_blue());
            }
        }
        "temp-path" | "temp_path" => {
            let path = PathBuf::from(&value);

            if path.exists() {
                database.temp_path = path;
                database.settings_mut().temp_path = value;
            } else {
                error!(target: "config", "Setting of {} is not committed, supplied path does not exist", "temp-path".bright_blue());
            }
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
        vfmt::println!("Usage:");
        vfmt::println!("  varela config push <KEY> <VALUE>");
        vfmt::println!();
        vfmt::println!("Options:");
        vfmt::println!("  --help");
        vfmt::println!();
        vfmt::println!("Arguments:");
        vfmt::println!("  key             the list to add to");
        vfmt::println!("  value           the value to add to the list");

        return Ok(());
    }

    let key = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("`config push` is missing `key` value"))?;
    let value = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("`config push` is missing `value` value"))?;

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
        vfmt::println!("Usage:");
        vfmt::println!("  varela config pop <KEY>");
        vfmt::println!();
        vfmt::println!("Options:");
        vfmt::println!("  --help");
        vfmt::println!();
        vfmt::println!("Arguments:");
        vfmt::println!("  key             the list item to remove");

        return Ok(());
    }

    let key = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("`config pop` is missing `key` value"))?;

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
