use std::{collections::HashMap, sync::RwLock};

use common::{
    database::Database,
    models::{Entity, Existing, Id, Index, ResolvedStory, ResolvedStoryMeta, StoryInfo, StoryMeta},
    prelude::*,
};
use enrgy::http::HttpResponse;
use once_cell::sync::Lazy;

pub fn wrap<F>(fun: F) -> HttpResponse
where
    F: FnOnce() -> Result<HttpResponse>,
{
    match fun() {
        Ok(res) => res,
        Err(err) => {
            error!("handler error: {}", err);

            HttpResponse::internal_server_error()
        }
    }
}

pub mod http {
    use {
        common::prelude::*,
        std::{fs, path::Path},
    };

    pub fn get<P>(temp_path: P, url: &str) -> Result<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        let url = url.replace(
            &['\\', '/', ':', ';', '<', '>', '"', '|', '?', '*', '[', ']'][..],
            "-",
        );

        let temp_path = temp_path.as_ref();

        fs::create_dir_all(&temp_path)?;

        let temp_file_path = temp_path.join(&url);

        let output = common::utils::command("curl")
            .arg("-L")
            .arg("-o")
            .arg(&temp_file_path)
            .arg(&url)
            .output()?;

        if !output.status.success() {
            let mut err = anyhow!("curl return with error code {:?}", output.status);

            if let Ok(text) = String::from_utf8(output.stdout) {
                err = err.context(format!("with stdout: {}", text));
            }

            if let Ok(text) = String::from_utf8(output.stderr) {
                err = err.context(format!("with stderr: {}", text));
            }

            return Err(err);
        }

        let bytes = fs::read(&temp_file_path)?;

        fs::remove_file(&temp_file_path)?;

        Ok(bytes)
    }
}

static STORY_CACHE: Lazy<RwLock<HashMap<Id, ResolvedStory>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[allow(clippy::ptr_arg)]
pub fn get_story_full(db: &Database, id: &Id) -> Result<ResolvedStory> {
    if let Some(story) = STORY_CACHE
        .read()
        .map_err(|err| anyhow!("unable to get lock on cache: {}", err))?
        .get(id)
        .cloned()
    {
        return Ok(story);
    }

    enum Kind {
        Categories,
        Authors,
        Origins,
        Warnings,
        Pairings,
        Characters,
        Generals,
    }

    fn values(index: &Index, meta: &StoryMeta, kind: &Kind) -> Result<Vec<Existing<Entity>>> {
        let (map, keys) = match kind {
            Kind::Categories => (&index.categories, &meta.categories),
            Kind::Authors => (&index.authors, &meta.authors),
            Kind::Origins => (&index.origins, &meta.origins),
            Kind::Warnings => (&index.warnings, &meta.warnings),
            Kind::Pairings => (&index.pairings, &meta.pairings),
            Kind::Characters => (&index.characters, &meta.characters),
            Kind::Generals => (&index.generals, &meta.generals),
        };

        keys.iter()
            .map(|id| {
                map.get(id)
                    .cloned()
                    .map(|entity| Existing::new(id.clone(), entity))
                    .ok_or_else(|| anyhow!("entity with id `{}` does not exist", id))
            })
            .collect::<Result<Vec<_>>>()
    }

    let index = db.index();

    let story_ref = index
        .stories
        .get(id)
        .ok_or_else(|| anyhow!("story with id `{}` does not exist", id))?;

    let info = &story_ref.info;
    let meta = &story_ref.meta;

    let story = ResolvedStory {
        chapters: story_ref.chapters.clone(),
        site: story_ref.site,
        info: StoryInfo {
            file_name: story_ref.info.file_name.clone(),
            file_hash: story_ref.info.file_hash,
            kind: story_ref.info.kind,
            title: info.title.clone(),
            summary: info.summary.clone(),
            created: story_ref.info.created.clone(),
            updated: story_ref.info.updated.clone(),
        },
        meta: ResolvedStoryMeta {
            rating: meta.rating,
            categories: values(index, meta, &Kind::Categories).context("categories")?,
            authors: values(index, meta, &Kind::Authors).context("authors")?,
            origins: values(index, meta, &Kind::Origins).context("origins")?,
            warnings: values(index, meta, &Kind::Warnings).context("warnings")?,
            pairings: values(index, meta, &Kind::Pairings).context("pairings")?,
            characters: values(index, meta, &Kind::Characters).context("characters")?,
            generals: values(index, meta, &Kind::Generals).context("generals")?,
        },
    };

    STORY_CACHE
        .write()
        .map_err(|err| anyhow!("unable to get lock on cache: {}", err))?
        .insert(id.clone(), story.clone());

    Ok(story)
}

pub struct Readable<N>
where
    N: std::fmt::Display,
{
    inner: N,
}

impl<N> std::fmt::Display for Readable<N>
where
    N: std::fmt::Display,
{
    #[allow(clippy::needless_collect)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values: Vec<(Option<char>, char)> = self
            .inner
            .to_string()
            .chars()
            .rev()
            .enumerate()
            .map(|(i, c)| {
                (
                    if i % 3 == 0 && i != 0 {
                        Some(',')
                    } else {
                        None
                    },
                    c,
                )
            })
            .collect();

        for (s, c) in values.into_iter().rev() {
            write!(f, "{}", c)?;

            if let Some(c) = s {
                write!(f, "{}", c)?;
            }
        }

        Ok(())
    }
}

pub trait IntoReadable: std::fmt::Display + Sized {
    fn into_readable(self) -> Readable<Self> {
        Readable { inner: self }
    }
}

impl IntoReadable for usize {}
impl IntoReadable for isize {}

impl IntoReadable for u32 {}
impl IntoReadable for u64 {}

impl IntoReadable for i32 {}
impl IntoReadable for i64 {}
