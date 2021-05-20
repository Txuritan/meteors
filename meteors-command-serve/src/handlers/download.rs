use {
    crate::{
        router::{Context, Response},
        templates::{pages, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    std::fs,
};

pub fn download_get(ctx: Context<'_, Database>) -> Result<Response> {
    let db = ctx
        .database
        .read()
        .map_err(|err| anyhow!("Unable to get read lock on the database: {:?}", err))?;

    let query = ctx.rebuild_query();

    let body = Layout::new(
        Width::Slim,
        db.settings().theme(),
        "downloads",
        query.clone(),
        pages::Download::new(),
    );

    Ok(crate::res!(200; body))
}

pub fn download_post(mut ctx: Context<'_, Database>) -> Result<Response> {
    let body = {
        let len = ctx.length();

        let reader = ctx.as_reader();

        let mut buf = String::with_capacity(len);

        reader.read_to_string(&mut buf)?;

        buf
    };

    let mut parse = form_urlencoded::parse(body.as_bytes());

    if let Some((_, url)) = parse.find(|(key, _)| key == "download") {
        fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            haystack
                .windows(needle.len())
                .position(|window| window == needle)
        }

        let db = ctx
            .database
            .read()
            .map_err(|err| anyhow!("Unable to get read lock on the database: {:?}", err))?;

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

        let query = ctx.rebuild_query();

        let body = Layout::new(
            Width::Slim,
            db.settings().theme(),
            "downloads",
            query.clone(),
            pages::Download::new(),
        );

        Ok(crate::res!(200; body))
    } else {
        Ok(crate::res!(503))
    }
}
