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
        router::{get, post, Router},
    },
    std::{
        env,
        net::{Ipv4Addr, SocketAddr},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    },
    tiny_http::Server,
};

mod prelude {
    pub use {
        crate::utils::new_id,
        ::anyhow::{self, anyhow, Context as _, Result},
        log::{debug, error, info, trace, warn},
        owo_colors::OwoColorize as _,
    };
}

fn main() -> Result<()> {
    logger::init()?;

    let (help, cfg) = env::args().skip(1).fold(
        Ok((false, Config::new())),
        |init: Result<(bool, Config)>, arg| {
            init.and_then(|(mut help, mut cfg)| {
                match arg.as_str() {
                    "-ct" | "--tc" => {
                        cfg.compress = true;
                        cfg.trackers = true;
                    }
                    "-cp" | "--pc" => {
                        cfg.compress = true;
                        cfg.parent = true;
                    }
                    "-tp" | "--pt" => {
                        cfg.parent = true;
                        cfg.trackers = true;
                    }
                    "-cpt" | "-ctp" | "-pct" | "-ptc" | "-tcp" | "-tpc" => {
                        cfg.compress = true;
                        cfg.parent = true;
                        cfg.trackers = true;
                    }
                    "-c" | "--compress" => cfg.compress = true,
                    "-p" | "--parent" => cfg.parent = true,
                    "-t" | "--trackers" => cfg.trackers = true,
                    "-h" | "--help" => help = true,
                    "--host" => cfg.take = Take::Host,
                    "--port" => cfg.take = Take::Port,
                    "--key" => cfg.take = Take::Key,
                    _ if cfg.take == Take::Host => {
                        cfg.host = arg.parse()?;
                        cfg.take = Take::None;
                    }
                    _ if cfg.take == Take::Port => {
                        cfg.port = arg.parse()?;
                        cfg.take = Take::None;
                    }
                    _ if cfg.take == Take::Key => {
                        cfg.key = arg.to_string();
                        cfg.take = Take::None;
                    }
                    _ => {}
                }

                Ok((help, cfg))
            })
        },
    )?;

    if help {
        println!("meteors {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("USAGE:");
        println!("    meteors [FLAGS] [OPTIONS]");
        println!();
        println!("FLAGS:");
        println!("    -h, --help            Prints help information");
        println!("    -c, --compress        Enables the auto compression of data files");
        println!("    -p, --parent          Sets this server to be parent node");
        println!("    -t, --trackers        Enables the removal of trackers from data files");
        println!();
        println!("OPTIONS:");
        println!("    --host <ADDRESS>      Sets the server's bound IP address [default: 0.0.0.0]");
        println!("    --port <NUMBER>       Sets the port that the server will listen to requests on [default: 8723]");
        println!("    --key <TOKEN>         The token that children nodes will connect with");

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

    let database = Database::init(&cfg)?;

    let router = Arc::new(
        Router::new(database)
            .on("/", get(handlers::index))
            .on("/story/:id/:chapter", get(handlers::story))
            .on("/search", post(handlers::search)),
    );

    let server =
        Arc::new(Server::http(addr).map_err(|err| anyhow!("unable to start server: {}", err))?);

    info!(
        "{} sever listening on: {}",
        "+".bright_black(),
        addr.bright_purple()
    );

    let mut guards = Vec::with_capacity(4);

    for id in 0..4 {
        guards.push(thread::spawn({
            let stop = stop.clone();
            let router = router.clone();
            let server = server.clone();

            move || loop {
                match server.recv_timeout(Duration::from_millis(100)) {
                    Ok(Some(req)) => {
                        if let Err(err) = router.handle(req) {
                            error!("{} unable to handle request {:?}", "+".bright_black(), err);
                        }
                    }
                    Ok(None) => {
                        if stop.load(Ordering::SeqCst) {
                            info!("{} shutting down server thread {}", "+".bright_black(), id.bright_purple());

                            break;
                        }
                    }
                    Err(err) => {
                        error!("{} {:?}", "+".bright_black(), err);
                    }
                }
            }
        }));
    }

    for guard in guards {
        if guard.join().is_err() {
            error!("{} unable to join server thread", "+".bright_black());
        }
    }

    Ok(())
}

pub struct Config {
    pub host: Ipv4Addr,
    pub port: u16,

    pub compress: bool,
    pub trackers: bool,

    pub parent: bool,
    pub key: String,

    take: Take,
}

impl Config {
    fn new() -> Self {
        Self {
            host: Ipv4Addr::new(0, 0, 0, 0),
            port: 8723,

            compress: false,
            trackers: false,

            parent: true,
            key: "".to_string(),

            take: Take::None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Take {
    None,
    Host,
    Port,
    Key,
}
