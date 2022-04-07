use common::models::{Entity, Existing};

use crate::templates::partials::{Contrast, Link};

#[derive(opal::Template)]
#[template(path = "partials/origin-list.hbs")]
pub struct OriginList {
    pub origins: Vec<Existing<Entity>>,
}
