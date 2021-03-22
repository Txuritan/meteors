pub mod proto;

use {
    crate::models::proto::{Entity, Rating, StoryInfo},
    std::ops::Range,
};

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq)]
pub struct StoryFull {
    pub file_name: String,
    pub length: usize,
    pub chapters: Vec<Range<usize>>,

    pub info: StoryInfo,

    pub meta: StoryFullMeta,
}

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq)]
pub struct StoryFullMeta {
    pub rating: Rating,

    pub authors: Vec<Entity>,

    pub categories: Vec<Entity>,

    pub origins: Vec<Entity>,

    pub warnings: Vec<Entity>,
    pub pairings: Vec<Entity>,
    pub characters: Vec<Entity>,
    pub generals: Vec<Entity>,
}

impl Rating {
    pub const fn class(self) -> &'static str {
        match self {
            Rating::Explicit => "explicit",
            Rating::Mature => "mature",
            Rating::Teen => "teen",
            Rating::General => "general",
            Rating::NotRated => "not-rated",
            Rating::Unknown => "unknown",
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            Rating::Explicit => "e",
            Rating::Mature => "m",
            Rating::Teen => "t",
            Rating::General => "g",
            Rating::NotRated => "r",
            Rating::Unknown => "u",
        }
    }

    pub const fn to(self) -> i32 {
        match self {
            Rating::Explicit => 0,
            Rating::Mature => 1,
            Rating::Teen => 2,
            Rating::General => 3,
            Rating::NotRated => 4,
            Rating::Unknown => 5,
        }
    }

    pub const fn from(num: i32) -> Rating {
        match num {
            0 => Rating::Explicit,
            1 => Rating::Mature,
            2 => Rating::Teen,
            3 => Rating::General,
            4 => Rating::NotRated,
            _ => Rating::Unknown,
        }
    }
}

impl proto::Range {
    pub const fn from_std(range: Range<usize>) -> proto::Range {
        proto::Range {
            start: range.start as u32,
            end: range.end as u32,
        }
    }

    pub const fn to_std(&self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }
}
