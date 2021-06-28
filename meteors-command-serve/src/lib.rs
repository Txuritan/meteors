#![allow(incomplete_features)]
#![feature(const_generics)]

mod handlers;
mod templates;

mod filters;
mod router;
mod search;
mod utils;

use {
    common::{database::Database, prelude::*, Action},
    std::time::Instant,
    std::{
        net::{Ipv4Addr, SocketAddr},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    },
    tiny_http::Server,
    tiny_http_router::{get, post, Middleware, Router},
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

        let database = Arc::new({
            let mut db = Database::open()?;

            trace!(
                "{} with {} stories",
                "+".bright_black(),
                db.index().stories.len().bright_purple(),
            );

            db.lock_data()?;

            db
        });

        let router = Router::new()
            .data(database.clone())
            .service(get("/").to(handlers::index))
            .service(get("/download").to(handlers::download_get))
            .service(post("/download").to(handlers::download_post))
            .service(get("/story/:id/:chapter").to(handlers::story))
            .service(get("/search").to(handlers::search))
            .service(get("/search2").to(handlers::search_v2))
            .service(get("/style.css").to(handlers::style))
            .service(get("/favicon.ico").to(handlers::favicon))
            .wrap(LoggerMiddleware)
            .build();

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
                let router = router.clone();
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

        if let Ok(mut database) = Arc::try_unwrap(database) {
            database.unlock_data()?;
        } else {
            error!("unable to unwrap database, not unlocking data");
        }

        Ok(())
    }
}

struct LoggerMiddleware;

impl Middleware for LoggerMiddleware {
    fn before(&self, req: &mut tiny_http_router::HttpRequest) {
        use tiny_http_router::Method;

        let earlier = Instant::now();

        req.ext_mut().insert(earlier);

        fn to_colored_string(method: &Method) -> String {
            match method {
                Method::Get => format!("{}", "GET".green()),
                Method::Post => format!("{}", "POST".bright_blue()),
                Method::Put => format!("{}", "PUT".bright_purple()),
                Method::Patch => format!("{}", "PATCH".bright_yellow()),
                Method::Delete => format!("{}", "DELETE".bright_red()),
                Method::Head => "HEAD".to_owned(),
                Method::Connect => "CONNECT".to_owned(),
                Method::Options => "OPTION".to_owned(),
                Method::Trace => "TRACE".to_owned(),
            }
        }

        let url = req.url().to_string();

        let (url, _) = url.split_at(url.find('?').unwrap_or_else(|| url.len()));

        info!(
            target: "command_serve::router",
            "{} {} {}/{} {} {}",
            "+".bright_black(),
            "+".bright_black(),
            "HTTP".bright_yellow(),
            req.http_version(),
            to_colored_string(&req.method()),
            url.bright_purple(),
        );
    }

    fn after(&self, req: &tiny_http_router::HttpRequest, res: &tiny_http_router::HttpResponse) {
        let dur = req
            .ext()
            .get::<Instant>()
            .and_then(|earlier| {
                chrono::Duration::from_std(Instant::now().duration_since(*earlier)).ok()
            })
            .map(|dur| format!("{}", dur.num_milliseconds().bright_purple()))
            .unwrap_or_else(|| format!("{}", "??".bright_red()));

        info!(
            target: "command_serve::router",
            "{} {} {} {}ms",
            "+".bright_black(),
            "+".bright_black(),
            match res.status_code().0 {
                200 => format!("{}", "200".green()),
                404 => format!("{}", "404".bright_yellow()),
                503 => format!("{}", "503".bright_red()),
                code => format!("{}", code.to_string().bright_blue()),
            },
            dur,
        );
    }
}
