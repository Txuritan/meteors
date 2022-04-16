#![allow(incomplete_features)]
#![feature(adt_const_params, decl_macro, generic_const_exprs)]

mod handlers;
mod templates;

mod filters;
mod router;
mod search;
mod template;
mod utils;

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Instant,
};

use common::{database::Database, prelude::*};
use enrgy::{middleware::Middleware, route, App, HttpServer};

pub use self::router::res;

#[inline(never)]
pub fn run(mut args: common::Args) -> Result<()> {
    if args.peek().map(|a| a == "--help").unwrap_or_default() {
        println!("Usage:");
        println!("  varela serve <ARGS>");
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

        trace!("with {} stories", db.index().stories.len().bright_purple());

        db.lock_data()?;

        db
    });

    let server = HttpServer::new(
        App::new()
            .data(database.clone())
            .service(route::get("/").to(handlers::index))
            .service(route::get("/download").to(handlers::download_get))
            .service(route::post("/download").to(handlers::download_post))
            .service(route::get("/story/:id/:chapter").to(handlers::story))
            .service(route::get("/search").to(handlers::search))
            .service(route::get("/search2").to(handlers::search_v2))
            .service(route::get("/style.css").to(handlers::style))
            .service(route::get("/favicon.ico").to(handlers::favicon))
            .service(route::get("/author/:id").to(handlers::entity))
            .service(route::get("/origin/:id").to(handlers::entity))
            .service(route::get("/tag/:id").to(handlers::entity))
            .service(route::get("/opds/root.:ext").to(handlers::catalog))
            .default_service(route::to(|| -> enrgy::http::HttpResponse {
                crate::res!(404)
            }))
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

impl Middleware<enrgy::http::HttpRequest, enrgy::http::HttpResponse> for LoggerMiddleware {
    fn before(&self, req: &mut enrgy::http::HttpRequest) {
        use enrgy::http::HttpMethod;

        let earlier = Instant::now();

        req.extensions.insert(earlier);

        fn to_colored_string(method: &HttpMethod) -> String {
            match method {
                HttpMethod::Get => vfmt::format!("{}", "GET".green()),
                HttpMethod::Post => vfmt::format!("{}", "POST".bright_blue()),
                HttpMethod::Put => vfmt::format!("{}", "PUT".bright_purple()),
                HttpMethod::Patch => vfmt::format!("{}", "PATCH".bright_yellow()),
                HttpMethod::Delete => vfmt::format!("{}", "DELETE".bright_red()),
                HttpMethod::Head => "HEAD".to_owned(),
                HttpMethod::Connect => "CONNECT".to_owned(),
                HttpMethod::Options => "OPTION".to_owned(),
                HttpMethod::Trace => "TRACE".to_owned(),
            }
        }

        let path = req.uri.path.clone();

        info!(
            target: "command_serve::router",
            "{}/{} {} {}",
            "HTTP".bright_yellow(),
            req.version,
            to_colored_string(&req.method),
            path.bright_purple(),
        );
    }

    fn after(
        &self,
        req: &enrgy::http::HttpRequest,
        res: enrgy::http::HttpResponse,
    ) -> enrgy::http::HttpResponse {
        let dur = req
            .extensions
            .get::<Instant>()
            .and_then(|earlier| {
                chrono::Duration::from_std(Instant::now().duration_since(*earlier)).ok()
            })
            .map(|dur| vfmt::format!("{}", dur.num_milliseconds().bright_purple()))
            .unwrap_or_else(|| vfmt::format!("{}", "??".bright_red()));

        info!(
            target: "command_serve::router",
            "{} {}ms",
            match res.status.0 {
                200 => vfmt::format!("{}", "200".green()),
                404 => vfmt::format!("{}", "404".bright_yellow()),
                503 => vfmt::format!("{}", "503".bright_red()),
                code => vfmt::format!("{}", vfmt::uDisplay::to_string(&code).bright_blue()),
            },
            dur,
        );

        res
    }
}
