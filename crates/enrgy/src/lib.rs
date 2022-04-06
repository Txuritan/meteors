#![allow(incomplete_features)]
#![warn(nonstandard_style, rust_2018_idioms, future_incompatible)]
#![feature(
    adt_const_params,
    box_syntax,
    const_btree_new,
    const_maybe_uninit_assume_init,
    const_mut_refs,
    const_slice_from_raw_parts,
    const_trait_impl,
    decl_macro,
    generic_const_exprs,
    slice_ptr_get
)]

mod extractor;
mod utils;

mod app;
mod extensions;
mod handler;
mod responder;
mod route;
mod server;
mod service;

pub mod http;

pub mod error;
pub mod middleware;

pub use crate::{app::App, responder::Responder, server::HttpServer};

#[doc(inline)]
pub use crate::error::Error;

pub mod dev {
    pub use crate::{
        extensions::Extensions,
        service::{BoxedService, Service},
    };
}

pub mod web {
    pub use crate::{
        extractor::{
            Body, Data, Header, OptionalHeader, OptionalParam, OptionalQuery, Param, ParseHeader,
            ParseParam, ParseQuery, Query, RawQuery,
        },
        route::{connect, delete, get, head, options, patch, post, put, to, trace},
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
