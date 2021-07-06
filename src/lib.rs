#![allow(incomplete_features)]
#![feature(const_generics)]

mod extractor;

mod app;
mod extensions;
mod handler;
mod responder;
mod route;
mod server;
mod service;

pub mod http;
pub mod middleware;

pub mod error;

pub use crate::{app::App, responder::Responder, server::HttpServer};

#[doc(inline)]
pub use crate::{http::{request::HttpRequest, response::HttpResponse}, error::Error};

pub mod web {
    pub use crate::{
        extractor::{
            Body, Data, Header, OptionalHeader, OptionalParam, OptionalQuery, Param, Query,
            RawQuery,
        },
        route::{connect, delete, get, head, options, patch, post, put, trace},
    };
}

// A module for testing different route handlers.
// Mostly making sure they compile.
#[cfg(test)]
mod test_compile {
    use super::*;

    #[test]
    fn test_responder() {
        fn index() -> impl Responder {
            "Hello World!"
        }

        App::new().service(web::get("/").to(index));
    }

    #[test]
    fn test_str() {
        fn index() -> &'static str {
            "Hello World!"
        }

        App::new().service(web::get("/").to(index));
    }

    #[test]
    fn test_string() {
        fn index() -> String {
            "Hello World!".to_string()
        }

        App::new().service(web::get("/").to(index));
    }

    #[test]
    fn test_string_param() {
        fn index(name: web::Param<"name">) -> String {
            format!("Hello {}!", *name)
        }

        App::new().service(web::get("/:name").to(index));
    }

    #[derive(Debug)]
    struct TestError {}

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test Error")
        }
    }

    impl error::ResponseError for TestError {}

    #[test]
    fn test_error_string() {
        fn index() -> Result<String, TestError> {
            Ok("Hello World!".to_string())
        }

        App::new().service(web::get("/").to(index));
    }
}
