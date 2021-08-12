pub mod link;
pub mod nav;
pub mod origin_list;
pub mod pagination;
pub mod story;
pub mod tag_list;

pub use crate::templates::partials::{
    link::{Contrast, Link},
    nav::Nav,
    origin_list::OriginList,
    pagination::Pagination,
    story::StoryPartial,
    tag_list::TagList,
};
