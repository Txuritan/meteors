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
    pub const fn class(self) -> &'static str {
        match self {
            TagKind::Warning => "warning",
            TagKind::Pairing => "pairing",
            TagKind::Character => "character",
            TagKind::General => "general",
        }
    }
}
