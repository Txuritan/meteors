use common::{
    database::Database,
    models::{EntityKind, Id},
    prelude::*,
};
use enrgy::{extractor, response::IntoResponse};

use crate::{
    handlers::Template,
    templates::{pages, partials, Layout, Width},
    utils,
};

pub fn entity(
    db: extractor::Data<Database>,
    id: extractor::ParseParam<"id", Id>,
) -> Result<impl IntoResponse, pages::Error> {
    let kind = db
        .get_entity_from_id(&id)
        .ok_or(pages::Error::not_found())?;

    let entity = {
        let entities = match kind {
            EntityKind::Author => &db.index().authors,
            EntityKind::Warning => &db.index().warnings,
            EntityKind::Origin => &db.index().origins,
            EntityKind::Pairing => &db.index().pairings,
            EntityKind::Character => &db.index().characters,
            EntityKind::General => &db.index().generals,
        };

        unsafe { entities.get(&*id).unwrap_unchecked() }
    };

    let mut stories = db
        .index()
        .stories
        .iter()
        .filter(|(_, story)| {
            let entities = match kind {
                EntityKind::Author => &story.meta.authors,
                EntityKind::Warning => &story.meta.warnings,
                EntityKind::Origin => &story.meta.origins,
                EntityKind::Pairing => &story.meta.pairings,
                EntityKind::Character => &story.meta.characters,
                EntityKind::General => &story.meta.generals,
            };

            entities.contains(&id)
        })
        .map(|(id, _)| {
            utils::get_story_full(&db, id)
                .and_then(|story| partials::StoryPartial::new(id.clone(), story, None))
        })
        .collect::<Result<Vec<_>>>()?;

    stories.sort_by(|a, b| a.title().cmp(b.title()));

    Ok(Template(Layout::new(
        Width::Slim,
        db.settings().theme,
        &entity.text,
        None,
        pages::Index::new(stories),
    )))
}
