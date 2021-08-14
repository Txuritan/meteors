#![allow(incomplete_features)]
#![warn(nonstandard_style, rust_2018_idioms, future_incompatible)]
#![feature(
    box_syntax,
    const_btree_new,
    const_fn_trait_bound,
    const_generics,
    const_trait_impl,
    const_trait_bound_opt_out,
    const_mut_refs,
    option_result_unwrap_unchecked
)]

mod extractor;

mod app;
mod extensions;
mod handler;
mod responder;
mod route;
mod router;
mod server;
mod service;

pub mod http;
pub mod middleware;

pub mod error;

use std::collections::BTreeMap;

pub use crate::{app::App, responder::Responder, server::HttpServer};

#[doc(inline)]
pub use crate::{
    error::Error,
    http::{request::HttpRequest, response::HttpResponse},
};

pub mod web {
    pub use crate::{
        extractor::{
            Body, Data, Header, OptionalHeader, OptionalParam, OptionalQuery, Param, ParseHeader,
            ParseParam, ParseQuery, Query, RawQuery,
        },
        route::{connect, delete, get, head, options, patch, post, put, to, trace},
    };
}

// FIXME Remove this in the future
pub(crate) const fn new_btreemap<K: ?const Ord, V>() -> BTreeMap<K, V> {
    use std::mem::{forget, transmute};
    use std::cmp::Ordering;
    #[derive(PartialEq, Eq, PartialOrd)]
    #[transparent]
    struct ConstOrdWrapper<T>(T);

    impl<T: ?const Ord> const Ord for ConstOrdWrapper<T> {
        fn cmp(&self, _: &Self) -> Ordering {
            Ordering::Equal
        }
        fn max(self, s: Self) -> Self { 
            forget(s);
            self 
        }
        fn min(self, s: Self) -> Self { 
            forget(s);
            self 
        }
        fn clamp(self, a: Self, b: Self) -> Self { 
            forget(a);
            forget(b);
            self 
        } 
    }
    unsafe {
        transmute::<BTreeMap<ConstOrdWrapper<K>, V>, _>(BTreeMap::new())
    }
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
