use {
    aloene::Aloene,
    std::{collections::BTreeMap, ops::Range},
};

#[derive(Debug, Clone, Copy, PartialEq, Aloene)]
pub enum Site {
    ArchiveOfOurOwn,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    Epub,
    Html,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Entity {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Meteors {
    pub settings: Settings,
    pub index: Index,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Settings {
    /// the theme of the website (default `LIGHT`)
    pub theme: Theme,
    /// this instances sync key
    pub sync_key: String,
    /// other instances that this instance will try to sync with
    pub nodes: Vec<Node>,
}

/// Nested message and enum types in `Settings`.
#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Node {
    /// the name of an instance node, used to allow the user to identify which node is what
    pub name: String,
    /// the key of an instance node, this is used for verification
    pub key: String,
    /// the host address of an instance, used for communication with said instance node
    pub host: String,
    /// the port of an instance, used for port scanning to find said instance node
    pub port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Aloene)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_class(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Index {
    pub stories: BTreeMap<String, Story>,
    pub categories: BTreeMap<String, Entity>,
    pub authors: BTreeMap<String, Entity>,
    pub origins: BTreeMap<String, Entity>,
    pub warnings: BTreeMap<String, Entity>,
    pub pairings: BTreeMap<String, Entity>,
    pub characters: BTreeMap<String, Entity>,
    pub generals: BTreeMap<String, Entity>,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Story {
    pub file_name: String,
    pub file_hash: u64,
    pub info: StoryInfo,
    pub meta: StoryMeta,
    // pub site: Site,
    pub chapters: Vec<Chapter>,
}

/// Nested message and enum types in `Story`.
#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct StoryInfo {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct StoryMeta {
    pub rating: Rating,
    pub authors: Vec<String>,
    pub categories: Vec<String>,
    pub origins: Vec<String>,
    pub warnings: Vec<String>,
    pub pairings: Vec<String>,
    pub characters: Vec<String>,
    pub generals: Vec<String>,
}

/// Nested message and enum types in `Meta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Aloene)]
pub enum Rating {
    Explicit,
    Mature,
    Teen,
    General,
    NotRated,
    Unknown,
}

impl Rating {
    pub const fn name(self) -> &'static str {
        match self {
            Rating::Explicit => "Explicit",
            Rating::Mature => "Mature",
            Rating::Teen => "Teen",
            Rating::General => "General",
            Rating::NotRated => "Not Rated",
            Rating::Unknown => "Unknown",
        }
    }

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
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Chapter {
    pub title: String,
    pub content: Range<usize>,
    pub summary: Option<String>,
    pub start_notes: Option<Range<usize>>,
    pub end_notes: Option<Range<usize>>,
}

pub mod resolved {
    use super::{Entity, Rating};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Story {
        pub file_name: String,
        pub file_hash: u64,
        pub info: super::StoryInfo,
        pub meta: StoryMeta,
        pub chapters: Vec<super::Chapter>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct StoryMeta {
        pub rating: Rating,
        pub authors: Vec<Entity>,
        pub categories: Vec<Entity>,
        pub origins: Vec<Entity>,
        pub warnings: Vec<Entity>,
        pub pairings: Vec<Entity>,
        pub characters: Vec<Entity>,
        pub generals: Vec<Entity>,
    }
}
