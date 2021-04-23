mod handlers;

mod filters;
mod router;
mod search;
mod utils;
mod views;

use {
    crate::router::{get, Router},
    common::{database::Database, prelude::*},
    seahorse::{Command, Context, Flag, FlagType},
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

pub fn command() -> Command {
    Command::new("serve")
        .description("run the internal web server")
        .flag(
            Flag::new("host", FlagType::String)
                .description("Sets the server's bound IP address [default: 0.0.0.0]"),
        )
        .flag(Flag::new("port", FlagType::Int).description(
            "Sets the port that the server will listen to requests on [default: 8723]",
        ))
        .action(|ctx| {
            common::action("serve", ctx, run);
        })
}

fn run(ctx: &Context) -> Result<()> {
    let stop = Arc::new(AtomicBool::new(false));

    ctrlc::set_handler({
        let stop = Arc::clone(&stop);

        move || {
            stop.store(true, Ordering::SeqCst);
        }
    })?;

    let addr: SocketAddr = (
        ctx.string_flag("host")
            .unwrap_or_else(|_| String::from("0.0.0.0"))
            .parse::<Ipv4Addr>()?,
        ctx.int_flag("port").unwrap_or(8723) as u16,
    )
        .into();

    let database = Arc::new(RwLock::new({
        let mut db = Database::open()?;

        trace!(
            "{} with {} stories",
            "+".bright_black(),
            db.index.stories.len().bright_purple(),
        );

        db.lock_data()?;

        db
    }));

    let router = Arc::new(
        Router::new(database.clone())
            .on("/", get(handlers::index))
            .on("/story/:id/:chapter", get(handlers::story))
            .on("/search", get(handlers::search))
            .on("/style.css", get(handlers::style)),
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
