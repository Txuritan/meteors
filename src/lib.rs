#![allow(incomplete_features)]
#![feature(const_generics)]

mod extractor;
mod http;

mod app;
mod error;
mod extensions;
mod handler;
mod middleware;
mod responder;
mod route;
mod server;
mod service;

pub use crate::{
    app::App,
    error::Error,
    http::{HttpRequest, HttpResponse, Method, StatusCode},
    middleware::Middleware,
    responder::Responder,
    server::HttpServer,
};

pub mod web {
    pub use crate::{
        extractor::{
            Body, Data, Header, OptionalHeader, OptionalParam, OptionalQuery, Param, Query,
            RawQuery,
        },
        route::{get, post},
    };
}
