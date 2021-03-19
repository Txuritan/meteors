use {crate::database::Id, std::ops::Range};

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Story<M> {
    pub file_name: String,
    pub length: usize,
    pub chapters: Vec<Range<usize>>,

    pub info: StoryInfo,

    pub meta: M,
}

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct StoryInfo {
    pub title: String,
    pub summary: String,
}

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct StoryMetaRef {
    pub rating: Rating,

    pub authors: Vec<Id>,

    pub categories: Vec<Id>,

    pub origins: Vec<Id>,

    pub warnings: Vec<Id>,
    pub pairings: Vec<Id>,
    pub characters: Vec<Id>,
    pub generals: Vec<Id>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct StoryMetaFull {
    pub rating: Rating,

    pub authors: Vec<Entity>,

    pub categories: Vec<Entity>,

    pub origins: Vec<Entity>,

    pub warnings: Vec<Entity>,
    pub pairings: Vec<Entity>,
    pub characters: Vec<Entity>,
    pub generals: Vec<Entity>,
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Rating {
    Explicit,
    Mature,
    Teen,

    NotRated,

    Unknown,
}

impl Rating {
    pub fn class(&self) -> &'static str {
        match self {
            Rating::Explicit => "explicit",
            Rating::Mature => "mature",
            Rating::Teen => "teen",
            Rating::NotRated => "not-rated",
            Rating::Unknown => "unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Rating::Explicit => "e",
            Rating::Mature => "m",
            Rating::Teen => "t",
            Rating::NotRated => "r",
            Rating::Unknown => "u",
        }
    }
}

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Entity {
    pub text: String,
}
