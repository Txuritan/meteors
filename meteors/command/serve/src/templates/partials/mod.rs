pub mod link;
pub mod nav;
pub mod origin_list;
pub mod story;
pub mod tag_list;

pub use crate::templates::partials::{
    link::{Contrast, Link},
    nav::Nav,
    origin_list::OriginList,
    story::StoryPartial,
    tag_list::TagList,
};
