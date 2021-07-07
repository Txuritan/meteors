use {crate::templates::TagKind, common::models::Entity};

#[derive(opal::Template)]
#[template(path = "partials/tag-list.hbs")]
pub struct TagList {
    pub tags: Vec<(TagKind, Entity)>,
}
