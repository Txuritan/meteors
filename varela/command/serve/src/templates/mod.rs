pub mod pages;
pub mod partials;

pub mod layout;

pub use crate::templates::layout::Layout;

pub enum Width {
    Slim,
    Wide,
}

impl Width {
    fn as_class(&self) -> &'static str {
        match self {
            Width::Slim => "slim",
            Width::Wide => "wide",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TagKind {
    Warning,
    Pairing,
    Character,
    General,
}

impl TagKind {
    pub const fn classes(self) -> &'static str {
        match self {
            TagKind::Warning => "bg-red-400 hover:bg-red-500",
            TagKind::Pairing => "bg-yellow-400 hover:bg-yellow-500",
            TagKind::Character => "bg-blue-400 hover:bg-blue-500",
            TagKind::General => "bg-gray-400 hover:bg-gray-500",
        }
    }
}
