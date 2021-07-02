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
    extractor::{Body, Data, OptionalHeader, OptionalQuery, Param, Query, RawQuery},
    http::{HttpRequest, HttpResponse, Method, StatusCode},
    middleware::Middleware,
    route::{get, post},
    server::HttpServer,
};
