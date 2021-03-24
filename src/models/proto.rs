use {
    nanoserde::{SerJson, SerJsonState},
    std::collections::BTreeMap,
};

#[derive(Clone, PartialEq, prost::Message, SerJson)]
pub struct Index {
    #[nserde(proxy = "StoryMapProxy")]
    #[prost(btree_map = "string, message", tag = "1")]
    pub stories: BTreeMap<String, Story>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "2")]
    pub categories: BTreeMap<String, Entity>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "3")]
    pub authors: BTreeMap<String, Entity>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "4")]
    pub origins: BTreeMap<String, Entity>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "5")]
    pub warnings: BTreeMap<String, Entity>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "6")]
    pub pairings: BTreeMap<String, Entity>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "7")]
    pub characters: BTreeMap<String, Entity>,
    #[nserde(proxy = "EntityMapProxy")]
    #[prost(btree_map = "string, message", tag = "8")]
    pub generals: BTreeMap<String, Entity>,
}

#[derive(Clone, PartialEq, prost::Message, SerJson)]
pub struct Range {
    #[prost(uint32, tag = "1")]
    pub start: u32,
    #[prost(uint32, tag = "2")]
    pub end: u32,
}

#[derive(Clone, PartialEq, prost::Message, SerJson)]
pub struct Story {
    #[prost(string, tag = "1")]
    pub file_name: String,
    #[prost(uint32, tag = "2")]
    pub length: u32,
    #[prost(message, repeated, tag = "3")]
    pub chapters: Vec<Range>,
    #[prost(message, required, tag = "4")]
    pub info: StoryInfo,
    #[prost(message, required, tag = "5")]
    pub meta: StoryMeta,
}

#[derive(Clone, PartialEq, prost::Message, SerJson)]
pub struct StoryInfo {
    #[prost(string, tag = "1")]
    pub title: String,
    #[prost(string, tag = "2")]
    pub summary: String,
}

#[derive(Clone, PartialEq, prost::Message, SerJson)]
pub struct StoryMeta {
    #[nserde(proxy = "RatingProxy")]
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

#[derive(Clone, PartialEq, prost::Message, SerJson)]
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

struct StoryMapProxy<'m> {
    inner: MapProxy<'m, Story>,
}

impl<'m> From<&'m BTreeMap<String, Story>> for StoryMapProxy<'m> {
    fn from(inner: &'m BTreeMap<String, Story>) -> Self {
        Self {
            inner: MapProxy::from(inner),
        }
    }
}

impl<'m> SerJson for StoryMapProxy<'m> {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        SerJson::ser_json(&self.inner, d, s)
    }
}

struct EntityMapProxy<'m> {
    inner: MapProxy<'m, Entity>,
}

impl<'m> From<&'m BTreeMap<String, Entity>> for EntityMapProxy<'m> {
    fn from(inner: &'m BTreeMap<String, Entity>) -> Self {
        Self {
            inner: MapProxy::from(inner),
        }
    }
}

impl<'m> SerJson for EntityMapProxy<'m> {
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        SerJson::ser_json(&self.inner, d, s)
    }
}

struct MapProxy<'m, V> {
    inner: &'m BTreeMap<String, V>,
}

impl<'m> From<&'m BTreeMap<String, Story>> for MapProxy<'m, Story> {
    fn from(inner: &'m BTreeMap<String, Story>) -> Self {
        Self { inner }
    }
}

impl<'m> From<&'m BTreeMap<String, Entity>> for MapProxy<'m, Entity> {
    fn from(inner: &'m BTreeMap<String, Entity>) -> Self {
        Self { inner }
    }
}

impl<'m, V> SerJson for MapProxy<'m, V>
where
    V: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('{');

        let len = self.inner.len();

        for (index, (k, v)) in self.inner.iter().enumerate() {
            s.indent(d + 1);

            k.ser_json(d + 1, s);

            s.out.push(':');

            v.ser_json(d + 1, s);

            if (index + 1) < len {
                s.conl();
            }
        }

        s.indent(d);

        s.out.push('}');
    }
}

struct RatingProxy<'m> {
    inner: &'m i32,
}

impl<'m> From<&'m i32> for RatingProxy<'m> {
    fn from(inner: &'m i32) -> Self {
        Self { inner }
    }
}

impl<'m> SerJson for RatingProxy<'m> {
    fn ser_json(&self, _: usize, s: &mut SerJsonState) {
        match self.inner {
            0 => {
                s.label("Explicit");
            }
            1 => {
                s.label("Mature");
            }
            2 => {
                s.label("Teen");
            }
            3 => {
                s.label("General");
            }
            4 => {
                s.label("NotRated");
            }
            _ => {
                s.label("Unknown");
            }
        }
    }
}
