use {crate::templates::TagKind, common::models::Entity};

pub struct TagList {
    pub tags: Vec<(TagKind, Entity)>,
}

include!("tag_list.hbs.rs");
