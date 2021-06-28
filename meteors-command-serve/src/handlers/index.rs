use {
    crate::{
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    tiny_http_router::{Data, Header, HttpResponse},
};

pub fn index(db: Data<Database>) -> Result<HttpResponse> {
    let mut stories = db
        .index()
        .stories
        .keys()
        .map(|id| {
            utils::get_story_full(&*db, id)
                .and_then(|(id, story)| partials::StoryCard::new(id, story, "".into()))
        })
        .collect::<Result<Vec<partials::StoryCard<'_>>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    let body = Layout::new(
        Width::Slim,
        db.settings().theme,
        "home",
        "".into(),
        pages::Index::new(stories),
    );

    Ok(crate::res!(200; body))
}

pub fn favicon() -> Result<HttpResponse> {
    Ok(
        HttpResponse::from_data(Vec::from(&include_bytes!("../../../assets/noel.ico")[..]))
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"image/x-icon"[..]).unwrap())
            .with_status_code(200),
    )
}
