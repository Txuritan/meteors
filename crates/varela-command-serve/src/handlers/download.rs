use {
    crate::{
        templates::{pages, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    enrgy::{http::HttpResponse, web},
    std::fs,
};

pub fn download_get(db: web::Data<Database>) -> HttpResponse {
    utils::wrap(|| {
        let body = Layout::new(
            Width::Slim,
            db.settings().theme,
            "downloads",
            None,
            pages::Download::new(),
        );

        Ok(crate::res!(200; body))
    })
}

pub fn download_post(db: web::Data<Database>, body: web::Body) -> HttpResponse {
    utils::wrap(|| {
        let mut parse = form_urlencoded::parse(&body);

        if let Some((_, url)) = parse.find(|(key, _)| key == "download") {
            fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
                haystack
                    .windows(needle.len())
                    .position(|window| window == needle)
            }

            let bytes = utils::http::get(&db.temp_path, &url)?;

            static TARGET_START: &[u8; 7] = b"<title>";
            static TARGET_END: &[u8; 8] = b"</title>";

            let start = find_subsequence(&bytes[..], &TARGET_START[..])
                .ok_or_else(|| anyhow!("Unable to find start of page title"))?;
            let end = find_subsequence(&bytes[..], &TARGET_END[..])
                .ok_or_else(|| anyhow!("Unable to find end of page title"))?;

            let whole_title = &bytes[(start + TARGET_START.len())..end];

            let first_dash = find_subsequence(whole_title, b" - ")
                .ok_or_else(|| anyhow!("Unable to find title separator"))?;

            let title = &whole_title[0..first_dash];

            let save_path = db
                .data_path
                .join(format!("{}.html", String::from_utf8(title.to_vec())?));

            fs::write(save_path, bytes)?;

            let body = Layout::new(
                Width::Slim,
                db.settings().theme,
                "downloads",
                None,
                pages::Download::new(),
            );

            Ok(crate::res!(200; body))
        } else {
            Ok(crate::res!(503))
        }
    })
}
