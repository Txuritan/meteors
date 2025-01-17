use common::{database::Database, models::Id, prelude::*};
use enrgy::{extractor, response::IntoResponse};

use crate::{
    handlers::Template,
    templates::{pages, partials, Layout, Width},
    utils,
};

pub struct IdParam;

impl enrgy::extractor::param::ParamKey for IdParam {
    const KEY: &'static str = "id";
}

pub struct ChapterParam;

impl enrgy::extractor::param::ParamKey for ChapterParam {
    const KEY: &'static str = "chapter";
}

pub fn story(
    db: extractor::Data<Database>,
    id: extractor::ParseParam<IdParam, Id>,
    index: extractor::Param<ChapterParam>,
) -> Result<impl IntoResponse, pages::Error> {
    let index: usize = index.parse().map_err(anyhow::Error::from)?;

    let story = utils::get_story_full(&db, &id)?;
    let chapter = db.get_chapter_body(&id, index)?;

    let body = Layout::new(
        Width::Slim,
        db.settings().theme,
        story.info.title.clone(),
        None,
        pages::Chapter::new(
            partials::StoryPartial::new(id.clone(), story, None)?,
            chapter,
            index,
        ),
    );

    Ok(Template(body))
}
