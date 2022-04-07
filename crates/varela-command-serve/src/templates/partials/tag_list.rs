use common::models::{Entity, Existing};

use crate::templates::TagKind;

#[derive(opal::Template)]
#[template(path = "partials/tag-list.hbs")]
pub struct TagList {
    pub tags: Vec<(TagKind, Existing<Entity>)>,
}
