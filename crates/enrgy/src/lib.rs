#![allow(
    incomplete_features,
    stable_features, // remove once cross updates
)]
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
    slice_ptr_get,
    // these two have to be here until cross publishes a new version
    const_fn_trait_bound,
    const_ptr_offset,
)]

#[macro_use]
mod utils;

mod app;
mod extensions;
mod handler;
mod server;
mod service;

pub mod extractor;
pub mod http;

pub mod error;
pub mod middleware;
pub mod response;
pub mod route;

pub use crate::{app::App, server::HttpServer};

#[doc(inline)]
pub use crate::error::Error;

pub mod dev {
    pub use crate::{
        extensions::Extensions,
        service::{BoxedService, Service},
    };
}

// A module for testing different route handlers.
// Mostly making sure they compile.
#[cfg(test)]
mod test_compile {
    use super::*;

    #[test]
    fn test_responder() {
        fn index() -> impl response::IntoResponse {
            "Hello World!"
        }

        App::new().service(route::get("/").to(index));
    }

    #[test]
    fn test_str() {
        fn index() -> &'static str {
            "Hello World!"
        }

        App::new().service(route::get("/").to(index));
    }

    #[test]
    fn test_string() {
        fn index() -> String {
            "Hello World!".to_string()
        }

        App::new().service(route::get("/").to(index));
    }

    #[test]
    fn test_string_param() {
        fn index(name: extractor::Param<"name">) -> String {
            format!("Hello {}!", *name)
        }

        App::new().service(route::get("/:name").to(index));
    }

    #[derive(Debug)]
    struct TestError {}

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test Error")
        }
    }

    impl response::IntoResponse for TestError {
        fn into_response(self) -> http::HttpResponse {
            http::HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }

    #[test]
    fn test_error_string() {
        fn index() -> Result<String, TestError> {
            Ok("Hello World!".to_string())
        }

        App::new().service(route::get("/").to(index));
    }
}
