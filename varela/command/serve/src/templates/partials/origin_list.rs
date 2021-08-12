use {
    crate::templates::partials::{Contrast, Link},
    common::models::{Entity, Existing},
};

#[derive(opal::Template)]
#[template(path = "partials/origin-list.hbs")]
pub struct OriginList {
    pub origins: Vec<Existing<Entity>>,
}
