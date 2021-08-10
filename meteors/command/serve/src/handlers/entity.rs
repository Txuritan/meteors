use {
    crate::{
        templates::{pages, partials, Layout, Width},
        utils,
    },
    common::{database::Database, prelude::*},
    enrgy::{web, HttpResponse},
};

enum EntityKind {
    Author,
    Warning,
    Origin,
    Pairing,
    Character,
    General,
}

macro impl_entity_handlers($( $name:ident => $kind:expr , )*) {
    $(
        pub fn $name(db: web::Data<Database>, id: web::Param<"id">) -> HttpResponse {
            inner(db, $kind, id)
        }
    )*
}

impl_entity_handlers! {
    author => EntityKind::Author,
    warning => EntityKind::Warning,
    origin => EntityKind::Origin,
    pairing => EntityKind::Pairing,
    character => EntityKind::Character,
    general => EntityKind::General,
}

fn inner(db: web::Data<Database>, kind: EntityKind, id: web::Param<"id">) -> HttpResponse {
    utils::wrap(|| {
        let entity = {
            let entities = match kind {
                EntityKind::Author => &db.inner.index.authors,
                EntityKind::Warning => &db.inner.index.warnings,
                EntityKind::Origin => &db.inner.index.origins,
                EntityKind::Pairing => &db.inner.index.pairings,
                EntityKind::Character => &db.inner.index.characters,
                EntityKind::General => &db.inner.index.generals,
            };

            entities.get(&*id)
        };

        if let Some(entity) = entity {
            let mut stories = db
                .inner
                .index
                .stories
                .iter()
                .filter(|(id, story)| {
                    let entities = match kind {
                        EntityKind::Author => &story.meta.authors,
                        EntityKind::Warning => &story.meta.warnings,
                        EntityKind::Origin => &story.meta.origins,
                        EntityKind::Pairing => &story.meta.pairings,
                        EntityKind::Character => &story.meta.characters,
                        EntityKind::General => &story.meta.generals,
                    };

                    entities.contains(id)
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
            Ok(crate::res!(404))
        }
    })
}
