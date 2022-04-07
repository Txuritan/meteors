use {
    crate::{
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{
        database::Database,
        models::{EntityKind, Id},
        prelude::*,
    },
    enrgy::{http::HttpResponse, web},
};

pub fn entity(db: web::Data<Database>, id: web::ParseParam<"id", Id>) -> HttpResponse {
    utils::wrap(|| {
        if let Some(kind) = db.get_entity_from_id(&*id) {
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

                    entities.contains(&*id)
                })
                .map(|(id, _)| {
                    utils::get_story_full(&db, id)
                        .and_then(|story| partials::StoryPartial::new(id, story, None))
                })
                .collect::<Result<Vec<_>>>()?;

            stories.sort_by(|a, b| a.title().cmp(b.title()));

            let body = Layout::new(
                Width::Slim,
                db.settings().theme,
                &entity.text,
                None,
                pages::Index::new(stories),
            );

            Ok(crate::res!(200; body))
        } else {
            debug!("entity with id `{}` does not exist", (&*id).bright_purple());

            Ok(crate::res!(404))
        }
    })
}
