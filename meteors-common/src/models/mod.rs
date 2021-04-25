pub mod proto;

pub use crate::models::proto::{story::meta::Rating, Entity, Range};

#[derive(Clone, PartialEq)]
pub struct Story {
    pub file_name: String,
    pub file_hash: u64,
    pub info: story::Info,
    pub meta: story::Meta,
    pub chapters: Vec<story::Chapter>,
}

pub mod story {
    use crate::models::{Entity, Rating};

    pub use crate::models::proto::story::{meta, Chapter, Info};

    #[derive(Clone, PartialEq)]
    pub struct Meta {
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
