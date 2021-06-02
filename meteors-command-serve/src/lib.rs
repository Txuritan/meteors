mod handlers;
mod templates;

mod filters;
mod router;
mod search;
mod utils;

use {
    crate::router::{get, post, Router},
    common::{database::Database, prelude::*, Action},
    std::{
        net::{Ipv4Addr, SocketAddr},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, RwLock,
        },
        thread,
        time::Duration,
    },
    tiny_http::Server,
};

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "serve",
    description = "run the internal web server"
)]
pub struct Command {
    #[argh(
        option,
        default = r#""0.0.0.0".to_owned()"#,
        description = "sets the server's bound IP address"
    )]
    host: String,
    #[argh(
        option,
        default = "8723",
        description = "sets the port that the server will listen to requests on"
    )]
    port: u16,
}

impl Action for Command {
    fn run(&self) -> Result<()> {
        let stop = Arc::new(AtomicBool::new(false));

        ctrlc::set_handler({
            let stop = Arc::clone(&stop);

            move || {
                stop.store(true, Ordering::SeqCst);
            }
        })?;

        let addr: SocketAddr = (self.host.parse::<Ipv4Addr>()?, self.port).into();

        let database = Arc::new(RwLock::new({
            let mut db = Database::open()?;

            trace!(
                "{} with {} stories",
                "+".bright_black(),
                db.index().stories.len().bright_purple(),
            );

            db.lock_data()?;

            db
        }));

        let router = Arc::new(
            Router::new(database.clone())
                .on("/", get(handlers::index))
                .on("/download", get(handlers::download_get))
                .on("/download", post(handlers::download_post))
                .on("/story/:id/:chapter", get(handlers::story))
                .on("/search", get(handlers::search))
                .on("/search2", get(handlers::search_v2))
                .on("/style.css", get(handlers::style))
                .on("/favicon.ico", get(handlers::favicon)),
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
                let stop = Arc::clone(&stop);
                let router = Arc::clone(&router);
                let server = Arc::clone(&server);

                move || loop {
                    match server.recv_timeout(Duration::from_millis(100)) {
                        Ok(Some(req)) => {
                            if let Err(err) = router.handle(req) {
                                error!("{} unable to handle request {:?}", "+".bright_black(), err);
                            }
                        }
                        Ok(None) => {
                            if stop.load(Ordering::SeqCst) {
                                info!(
                                    "{} shutting down server thread {}",
                                    "+".bright_black(),
                                    id.bright_purple()
                                );

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

        let mut db = database
            .write()
            .map_err(|err| anyhow!("Unable to get write lock on database: {:?}", err))?;

        db.unlock_data()?;

        Ok(())
    }
}
