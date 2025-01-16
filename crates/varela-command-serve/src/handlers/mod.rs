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
    extractor,
    http::HttpResponse,
    http::{
        self,
        headers::{CACHE_CONTROL, CONTENT_TYPE, ETAG},
    },
    response::{Html, IntoResponse},
};

pub struct Template<T>(pub T)
where
    T: opal::Template;

impl<T> IntoResponse for Template<T>
where
    T: opal::Template,
{
    fn into_response(self) -> HttpResponse {
        Html(unsafe { self.0.render_as_string().unwrap_unchecked() }).into_response()
    }
}

pub struct IfNoneMatchKey;

impl enrgy::extractor::header::HeaderKey for IfNoneMatchKey {
    const KEY: &'static str = "If-None-Match";
}

pub fn style(header: extractor::OptionalHeader<IfNoneMatchKey>) -> HttpResponse {
    static CSS: &str = include_str!("../../assets/dist/index.css");
    // RELEASE: change anytime theres a release and the style gets updated
    static CSS_TAG: &str = "\"C0017857370FA4EDD51C10B2276FEE04C76BDCA4879B415E510F958E4C3FF091\"";

    let mut res = HttpResponse::ok().header(CONTENT_TYPE, "text/css; charset=utf-8");

    if !cfg!(debug_assertions) {
        res = res
            .header(CACHE_CONTROL, "public; max-age=31536000")
            .header(ETAG, CSS_TAG);
    }

    if let Some(header) = header.as_deref() {
        if header == CSS_TAG {
            *res.status_mut() = http::StatusCode(304);
            return res;
        }
    }

    res.body(CSS)
}
