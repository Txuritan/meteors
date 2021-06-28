#![allow(incomplete_features)]
#![feature(const_generics)]

mod extractor;
mod handler;
mod http;
mod method;
mod middleware;
mod route;
mod router;
mod service;

pub use {
    crate::{
        extractor::{Body, Data, OptionalHeader, OptionalQuery, Param, Query, RawQuery},
        http::{HttpRequest, HttpResponse},
        method::Method,
        middleware::Middleware,
        route::{get, post},
        router::Router,
    },
    anyhow::Error,
    tiny_http::{Header, HeaderField, StatusCode},
};
