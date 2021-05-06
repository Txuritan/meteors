use common::models::Entity;

pub struct OriginList {
    pub origins: Vec<Entity>,
}

include!("origin_list.hbs.rs");
