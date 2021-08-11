#![allow(incomplete_features)]
#![feature(const_generics, decl_macro, option_result_unwrap_unchecked)]

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

pub use self::router::res;

#[inline(never)]
pub fn run(mut args: common::Args) -> Result<()> {
    // let stop = Arc::new(AtomicBool::new(false));

    // ctrlc::set_handler({
    //     let stop = Arc::clone(&stop);

    //     move || {
    //         stop.store(true, Ordering::SeqCst);
    //     }
    // })?;

    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        println!("Usage:");
        println!("  meteors serve <ARGS>");
        println!();
        println!("Options:");
        println!("  --help");
        println!();
        println!("Arguments:");
        println!("  host            sets the server's bound IP address [default: 0.0.0.0]");
        println!("  port            sets the port that the server will listen to requests on [default: 8723]");

        return Ok(());
    }

    let mut host = Ipv4Addr::new(0, 0, 0, 0);
    let mut port = 8723;

    for _ in 0..2 {
        match args.next().as_deref() {
            Some("--host") if args.peek().is_some() => {
                let value = unsafe { args.next().unwrap_unchecked() };

                host = value.parse::<Ipv4Addr>()?;
            }
            Some("--port") if args.peek().is_some() => {
                let value = unsafe { args.next().unwrap_unchecked() };

                port = value.parse::<u16>()?;
            }
            _ => {}
        }
    }

    let addr: SocketAddr = (host, port).into();

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
            .service(web::get("/tag/:id").to(handlers::entity))
            .service(web::get("/opds/root.:ext").to(handlers::catalog))
            .default_service(web::to(|| -> enrgy::HttpResponse { crate::res!(404) }))
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
