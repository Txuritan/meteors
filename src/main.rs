#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod data;
mod models;
mod regex;

mod handlers;
mod logger;
mod router;
mod utils;

use {
    crate::{
        data::Database,
        prelude::*,
        router::{get, Router},
    },
    std::{
        env,
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

    let cfg = env::args().skip(1).fold(Ok(Config::new()), |cfg: Result<Config>, arg| {
        cfg.and_then(|mut cfg| {
            match arg.as_str() {
                "-ct" | "--tc" => {
                    cfg.compress = true;
                    cfg.trackers = true;
                }
                "-c" | "--compress" => cfg.compress = true,
                "-t" | "--trackers" => cfg.trackers = true,
                "-h" | "--help" => cfg.help = true,
                "--host" => cfg.host_take = true,
                "--port" => cfg.port_take = true,
                _ if cfg.host_take => {
                    cfg.host = arg.parse()?;
                    cfg.host_take = false;
                }
                _ if cfg.port_take => {
                    cfg.port = arg.parse()?;
                    cfg.port_take = false;
                }
                _ => {}
            }

            Ok(cfg)
        })
    })?;

    if cfg.help {
        println!("meteors {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("USAGE:");
        println!("    meteors [FLAGS] [OPTIONS]");
        println!();
        println!("FLAGS:");
        println!("    -h, --help            Prints help information");
        println!("    -c, --compress        Enables the auto compression of data files");
        println!("    -t, --trackers        Enables the removal of trackers from data files");
        println!();
        println!("OPTIONS:");
        println!("    --host <ADDRESS>      Sets the server's bound IP address [default: 0.0.0.0]");
        println!("    --port <NUMBER>       Sets the port that the server will listen to requests on [default: 8723]");

        return Ok(());
    }
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

    pub help: bool,

    pub compress: bool,
    pub trackers: bool,

    host_take: bool,
    port_take: bool,
}

impl Config {
    const fn new() -> Self {
        Self {
            host: Ipv4Addr::new(0, 0, 0, 0),
            port: 8723,

            help: false,

            compress: false,
            trackers: false,

            host_take: false,
            port_take: false,
        }
    }
}
