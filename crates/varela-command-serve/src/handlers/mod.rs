mod download;
mod entity;
mod index;
mod opds;
mod search;
mod story;

pub use crate::handlers::{
    download::{download_get, download_post},
    entity::entity,
    index::{favicon, index},
    opds::catalog,
    search::{search, search_v2},
    story::story,
};

use enrgy::{
    http::HttpResponse,
    http::{
        self,
        headers::{CACHE_CONTROL, CONTENT_TYPE, ETAG},
    },
    web,
};

use crate::utils;

pub fn style(header: web::OptionalHeader<"If-None-Match">) -> HttpResponse {
    utils::wrap(|| {
        static CSS: &str = include_str!("../../assets/dist/index.css");
        // RELEASE: change anytime theres a release and the style gets updated
        static CSS_TAG: &str = "f621e1d55cbee8397c906c7d72d0fb9a4520a06be6218abeccff1ffcf75f00b3";

        let mut res = HttpResponse::ok().header(CONTENT_TYPE, "text/css; charset=utf-8");

        if !cfg!(debug_assertions) {
            res = res
                .header(CACHE_CONTROL, "public; max-age=31536000")
                .header(ETAG, CSS_TAG);
        }

        if let Some(header) = header.as_deref() {
            if header == CSS_TAG {
                return Ok(res.status(http::StatusCode(304)));
            }
        }

        Ok(res.body(CSS))
    })
}
