#![allow(incomplete_features)]
#![feature(const_generics)]

mod extractor;

mod app;
mod error;
mod extensions;
mod handler;
mod responder;
mod route;
mod server;
mod service;

pub mod http;
pub mod middleware;

pub use crate::{app::App, error::Error, responder::Responder, server::HttpServer};

#[doc(inline)]
pub use http::{request::HttpRequest, response::HttpResponse};

pub mod web {
    pub use crate::{
        extractor::{
            Body, Data, Header, OptionalHeader, OptionalParam, OptionalQuery, Param, Query,
            RawQuery,
        },
        route::{connect, delete, get, head, options, patch, post, put, trace},
    };
}
