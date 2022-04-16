use std::{borrow::Cow, collections::HashMap, ops::Range};

use aloene::Aloene;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(Cow<'static, str>);

impl Id {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Id(Cow::Owned(id))
    }
}

impl std::str::FromStr for Id {
    type Err = <String as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <String as std::str::FromStr>::from_str(s).map(Id::from)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl vfmt::uDisplay for Id {
    fn fmt<W>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: vfmt::uWrite + ?Sized,
    {
        self.0.fmt(f)
    }
}

impl Aloene for Id {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> Result<Self, aloene::Error> {
        <String as Aloene>::deserialize(reader).map(Id::from)
    }

    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), aloene::Error> {
        <String as Aloene>::serialize(&(self.0.as_ref().to_string()), writer)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Existing<T> {
    pub id: Id,

    inner: T,
}

impl<T> Existing<T> {
    pub fn new(id: Id, inner: T) -> Self {
        Self { id, inner }
    }
}

impl<T> std::ops::Deref for Existing<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Existing<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Aloene isn't actually used here, its just to satisfy trait bounds
impl<T> Aloene for Existing<T> {
    fn deserialize<R: std::io::Read>(_reader: &mut R) -> Result<Self, aloene::Error> {
        panic!("this should never be called")
    }

    fn serialize<W: std::io::Write>(&self, _writer: &mut W) -> Result<(), aloene::Error> {
        panic!("this should never be called")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Aloene)]
pub enum FileKind {
    Epub,
    Html,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Entity {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Config {
    pub version: Version,
    pub settings: Settings,
    pub index: Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Aloene)]
pub enum Version {
    V1,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Settings {
    /// the theme of the website (default `LIGHT`)
    pub theme: Theme,
    /// this instances sync key
    pub sync_key: String,
    /// the path to the folder that holds all the story files
    pub data_path: String,
    /// the path to a `temp` folder, used for downloads and epub reading
    pub temp_path: String,
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
    pub stories: HashMap<Id, Story>,
    pub categories: HashMap<Id, Entity>,
    pub authors: HashMap<Id, Entity>,
    pub origins: HashMap<Id, Entity>,
    pub warnings: HashMap<Id, Entity>,
    pub pairings: HashMap<Id, Entity>,
    pub characters: HashMap<Id, Entity>,
    pub generals: HashMap<Id, Entity>,
}

pub type Story = CoreStory<StoryMeta>;
pub type ResolvedStory = CoreStory<ResolvedStoryMeta>;

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct CoreStory<Meta>
where
    Meta: Aloene,
{
    pub info: StoryInfo,
    pub meta: Meta,
    pub site: Site,
    pub chapters: Vec<Chapter>,
}

/// Nested message and enum types in `Story`.
#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct StoryInfo {
    pub file_name: String,
    pub file_hash: u64,
    pub kind: FileKind,
    pub title: String,
    pub summary: String,
    pub created: String,
    pub updated: String,
}

pub type StoryMeta = StoryMetaCore<Id>;
pub type ResolvedStoryMeta = StoryMetaCore<Existing<Entity>>;

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct StoryMetaCore<Entity>
where
    Entity: Aloene,
{
    pub rating: Rating,
    pub authors: Vec<Entity>,
    pub categories: Vec<Entity>,
    pub origins: Vec<Entity>,
    pub warnings: Vec<Entity>,
    pub pairings: Vec<Entity>,
    pub characters: Vec<Entity>,
    pub generals: Vec<Entity>,
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

#[derive(Debug, Clone, Copy, PartialEq, Aloene)]
pub enum Site {
    ArchiveOfOurOwn,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Aloene)]
pub struct Chapter {
    pub title: String,
    pub content: Range<usize>,
    pub summary: Option<String>,
    pub start_notes: Option<Range<usize>>,
    pub end_notes: Option<Range<usize>>,
}

pub enum EntityKind {
    Author,
    Warning,
    Origin,
    Pairing,
    Character,
    General,
}
