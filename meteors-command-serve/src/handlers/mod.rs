mod download;
mod index;
mod search;
mod story;

pub use {
    crate::{
        handlers::{
            download::{download_get, download_post},
            index::{favicon, index},
            search::{search, search_v2},
            story::story,
        },
        router::{Context, Header, HeaderField, Response, StatusCode},
    },
    common::{database::Database, prelude::*},
    std::io::Cursor,
};

pub fn style(ctx: Context<'_, Database>) -> Result<Response> {
    static CSS: &str = include_str!("../../assets/style.css");
    // RELEASE: change anytime theres a release and the style gets updated
    static CSS_TAG: &str = "f621e1d55cbee8397c906c7d72d0fb9a4520a06be6218abeccff1ffcf75f00b3";

    let mut headers = Vec::with_capacity(16);

    headers
        .push(Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap());

    if !cfg!(debug_assertions) {
        headers.push(
            Header::from_bytes(&b"Cache-Control"[..], &b"public; max-age=31536000"[..]).unwrap(),
        );

        headers.push(Header::from_bytes(&b"ETag"[..], CSS_TAG).unwrap());
    }

    let target_header = HeaderField::from_bytes(&b"If-None-Match"[..])?;
    let header = ctx
        .headers()
        .iter()
        .find(|header| header.field == target_header);

    if let Some(header) = header {
        if header.value == CSS_TAG {
            return Ok(Response::new(
                StatusCode(304),
                headers,
                Cursor::new(vec![]),
                None,
                None,
            ));
        }
    }

    Ok(Response::new(
        StatusCode(200),
        headers,
        Cursor::new(CSS.as_bytes().to_vec()),
        Some(CSS.len()),
        None,
    ))
}
