use std::collections::BTreeMap;

#[derive(Clone, PartialEq, prost::Message)]
pub struct Index {
    #[prost(btree_map = "string, message", tag = "1")]
    pub stories: BTreeMap<String, Story>,
    #[prost(btree_map = "string, message", tag = "2")]
    pub categories: BTreeMap<String, Entity>,
    #[prost(btree_map = "string, message", tag = "3")]
    pub authors: BTreeMap<String, Entity>,
    #[prost(btree_map = "string, message", tag = "4")]
    pub origins: BTreeMap<String, Entity>,
    #[prost(btree_map = "string, message", tag = "5")]
    pub warnings: BTreeMap<String, Entity>,
    #[prost(btree_map = "string, message", tag = "6")]
    pub pairings: BTreeMap<String, Entity>,
    #[prost(btree_map = "string, message", tag = "7")]
    pub characters: BTreeMap<String, Entity>,
    #[prost(btree_map = "string, message", tag = "8")]
    pub generals: BTreeMap<String, Entity>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Range {
    #[prost(uint32, tag = "1")]
    pub start: u32,
    #[prost(uint32, tag = "2")]
    pub end: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Story {
    #[prost(string, tag = "1")]
    pub file_name: String,
    #[prost(uint64, tag = "2")]
    pub file_hash: u64,
    #[prost(uint32, tag = "3")]
    pub length: u32,
    #[prost(message, repeated, tag = "4")]
    pub chapters: Vec<Range>,
    #[prost(message, required, tag = "5")]
    pub info: StoryInfo,
    #[prost(message, required, tag = "6")]
    pub meta: StoryMeta,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct StoryInfo {
    #[prost(string, tag = "1")]
    pub title: String,
    #[prost(string, tag = "2")]
    pub summary: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct StoryMeta {
    #[prost(enumeration = "Rating", tag = "1")]
    pub rating: i32,
    #[prost(message, repeated, tag = "2")]
    pub authors: Vec<String>,
    #[prost(message, repeated, tag = "3")]
    pub categories: Vec<String>,
    #[prost(message, repeated, tag = "4")]
    pub origins: Vec<String>,
    #[prost(message, repeated, tag = "5")]
    pub warnings: Vec<String>,
    #[prost(message, repeated, tag = "6")]
    pub pairings: Vec<String>,
    #[prost(message, repeated, tag = "7")]
    pub characters: Vec<String>,
    #[prost(message, repeated, tag = "8")]
    pub generals: Vec<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Entity {
    #[prost(string, tag = "1")]
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum Rating {
    Explicit = 0,
    Mature = 1,
    Teen = 2,
    General = 3,
    NotRated = 4,
    Unknown = 5,
}
