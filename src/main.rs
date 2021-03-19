#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod regex;

mod database;
mod handlers;
mod logger;
mod models;
mod reader;
mod router;
mod search;
mod utils;

use {
    crate::{
        database::Database,
        prelude::*,
        router::{get, Router},
    },
    pico_args::Arguments,
    std::{
        net::{Ipv4Addr, SocketAddr},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    },
    tiny_http::Server,
};

mod prelude {
    pub use {
        crate::utils::new_id,
        color_eyre::{
            eyre::{self, eyre, Context as _},
            Help as _, Result,
        },
        log::{debug, error, info, trace, warn},
        owo_colors::OwoColorize as _,
    };
}

fn main() -> Result<()> {
    color_eyre::install()?;
    logger::init()?;

    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!(
            "{} {}",
            "meteors".green(),
            env!("CARGO_PKG_VERSION").bright_purple()
        );
        println!();
        println!("{}:", "USAGE".bright_yellow());
        println!("    {} [{}]", "meteors".green(), "OPTIONS".bright_yellow());
        println!();
        println!("{}:", "FLAGS".bright_yellow());
        println!(
            "    {}, {}               Prints help information",
            "-h".bright_black(),
            "--help".bright_black()
        );
        println!();
        println!("{}:", "OPTIONS".bright_yellow());
        println!(
            "    {} <IPV4_ADDRESS>    Sets the server's bound IP address [default: {}]",
            "--host".bright_black(),
            "0.0.0.0".bright_blue()
        );
        println!("    {} <NUMBER>          Sets the port that the server will listen to requests on [default: {}]", "--port".bright_black(), "8723".bright_purple());
        println!(
            "    {}               Enables the auto compression of data files",
            "--compress".bright_black()
        );
        println!(
            "    {}               Enables the removal of trackers from data files",
            "--trackers".bright_black()
        );

        return Ok(());
    }

    let cfg = Config {
        host: args
            .opt_value_from_str("--host")?
            .unwrap_or_else(|| Ipv4Addr::new(0, 0, 0, 0)),
        port: args.opt_value_from_str("--port")?.unwrap_or(8723),

        compress: args.contains("--compress"),
        trackers: args.contains("--trackers"),
    };

    let stop = Arc::new(AtomicBool::new(false));

    ctrlc::set_handler({
        let stop = stop.clone();

        move || {
            stop.store(true, Ordering::SeqCst);
        }
    })?;

    let addr: SocketAddr = (cfg.host, cfg.port).into();

    let database = Database::init(cfg)?;

    let mut router = Router::new(database)
        .on("/", get(handlers::index))
        .on("/story/:id/:chapter", get(handlers::story));

    let server = Server::http(addr).map_err(|err| eyre!("unable to start server: {}", err))?;

    info!(
        "{} sever listening on: {}",
        "+".bright_black(),
        addr.bright_purple()
    );

    loop {
        match server.recv_timeout(Duration::from_millis(100))? {
            Some(req) => router.handle(req)?,
            None => {
                if stop.load(Ordering::SeqCst) {
                    info!("{} shutting down server", "+".bright_black());

                    break;
                }
            }
        }
    }

    Ok(())
}

pub struct Config {
    pub host: Ipv4Addr,
    pub port: u16,

    pub compress: bool,
    pub trackers: bool,
}
