use common::models::Entity;

#[derive(opal::Template)]
#[template(path = "partials/origin-list.hbs")]
pub struct OriginList {
    pub origins: Vec<Entity>,
}
