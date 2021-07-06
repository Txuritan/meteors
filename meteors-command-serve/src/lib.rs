#![allow(incomplete_features)]
#![feature(const_generics)]

mod handlers;
mod templates;

mod filters;
mod router;
mod search;
mod utils;

use {
    common::{database::Database, prelude::*},
    enrgy::{middleware::Middleware, web, App, HttpServer},
    std::time::Instant,
    std::{
        net::{Ipv4Addr, SocketAddr},
        sync::Arc,
    },
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

#[inline(never)]
pub fn run(command: Command) -> Result<()> {
    // let stop = Arc::new(AtomicBool::new(false));

    // ctrlc::set_handler({
    //     let stop = Arc::clone(&stop);

    //     move || {
    //         stop.store(true, Ordering::SeqCst);
    //     }
    // })?;

    let addr: SocketAddr = (command.host.parse::<Ipv4Addr>()?, command.port).into();

    let database = Arc::new({
        let mut db = Database::open()?;

        trace!("with {} stories", db.index().stories.len().bright_purple(),);

        db.lock_data()?;

        db
    });

    let server = HttpServer::new(
        App::new()
            .data(database.clone())
            .service(web::get("/").to(handlers::index))
            .service(web::get("/download").to(handlers::download_get))
            .service(web::post("/download").to(handlers::download_post))
            .service(web::get("/story/:id/:chapter").to(handlers::story))
            .service(web::get("/search").to(handlers::search))
            .service(web::get("/search2").to(handlers::search_v2))
            .service(web::get("/style.css").to(handlers::style))
            .service(web::get("/favicon.ico").to(handlers::favicon))
            .wrap(LoggerMiddleware),
    )
    .bind(addr);

    info!("sever listening on: {}", addr.bright_purple());

    server.run()?;

    if let Ok(mut database) = Arc::try_unwrap(database) {
        database.unlock_data()?;
    } else {
        error!("unable to unwrap database, not unlocking data");
    }

    Ok(())
}

struct LoggerMiddleware;

impl Middleware for LoggerMiddleware {
    fn before(&self, req: &mut enrgy::HttpRequest) {
        use enrgy::http::Method;

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
            "{}/{} {} {}",
            "HTTP".bright_yellow(),
            req.version(),
            to_colored_string(&req.method()),
            url.bright_purple(),
        );
    }

    fn after(&self, req: &enrgy::HttpRequest, res: &enrgy::HttpResponse) {
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
            "{} {}ms",
            match res.status().0 {
                200 => format!("{}", "200".green()),
                404 => format!("{}", "404".bright_yellow()),
                503 => format!("{}", "503".bright_red()),
                code => format!("{}", code.to_string().bright_blue()),
            },
            dur,
        );
    }
}
