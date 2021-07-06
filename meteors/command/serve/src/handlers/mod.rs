mod download;
mod index;
mod search;
mod story;

pub use crate::handlers::{
    download::{download_get, download_post},
    index::{favicon, index},
    search::{search, search_v2},
    story::story,
};

use {
    crate::utils,
    enrgy::{http, web, HttpResponse},
};

pub fn style(header: web::OptionalHeader<"If-None-Match">) -> HttpResponse {
    utils::wrap(|| {
        static CSS: &str = include_str!("../../assets/style.css");
        // RELEASE: change anytime theres a release and the style gets updated
        static CSS_TAG: &str = "f621e1d55cbee8397c906c7d72d0fb9a4520a06be6218abeccff1ffcf75f00b3";

        let mut res = HttpResponse::ok().header("Content-Type", "text/css; charset=utf-8");

        if !cfg!(debug_assertions) {
            res = res
                .header("Cache-Control", "public; max-age=31536000")
                .header("ETag", CSS_TAG);
        }

        if let Some(header) = header.as_deref() {
            if header == CSS_TAG {
                return Ok(res.status(http::StatusCode(304)).finish());
            }
        }

        Ok(res.body(CSS))
    })
}
