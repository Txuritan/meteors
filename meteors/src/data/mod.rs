mod file;
mod reader;
pub mod search;

use {
    crate::{
        data::file::StoryFile,
        models::{
            proto::{Entity, Index, Range, Rating, StoryMeta},
            StoryFull, StoryFullMeta,
        },
        prelude::*,
        Config,
    },
    flate2::{read::GzDecoder, write::GzEncoder, Compression},
    prost::Message,
    std::{
        collections::BTreeMap,
        convert::TryInto as _,
        env,
        ffi::OsStr,
        fs::{self, DirEntry, File},
        io::{self, Read as _, Write as _},
        path::PathBuf,
    },
};

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq)]
pub struct Database {
    pub index: Index,

    pub children: Vec<String>,

    pub data_path: PathBuf,
    pub index_path: PathBuf,
}

impl Database {
    pub fn init(cfg: &Config) -> Result<Self> {
        debug!("{} building database", "+".bright_black());

        let cur = env::current_dir()?.canonicalize()?;

        let data_path = cur.join("data");
        let index_path = cur.join("index.pb.gz");

        fs::create_dir_all(&data_path)?;

        let mut database = if index_path.exists() {
            debug!("{} found existing", "|".bright_black());

            let mut decoder = GzDecoder::new(File::open(&index_path)?);

            let mut bytes = Vec::new();

            decoder.read_to_end(&mut bytes)?;

            let index = <Index as Message>::decode(&bytes[..])?;

            Self {
                index,

                children: vec![],

                data_path,
                index_path,
            }
        } else {
            debug!("{} not found, creating", "|".bright_black());

            Self {
                index: Index {
                    stories: BTreeMap::new(),
                    categories: BTreeMap::new(),
                    authors: BTreeMap::new(),
                    origins: BTreeMap::new(),
                    warnings: BTreeMap::new(),
                    pairings: BTreeMap::new(),
                    characters: BTreeMap::new(),
                    generals: BTreeMap::new(),
                },

                children: vec![],

                data_path,
                index_path,
            }
        };

        debug!(
            "{} {} checking data",
            "+".bright_black(),
            "+".bright_black(),
        );

        for entry in fs::read_dir(&database.data_path)? {
            let entry = entry?;
            let meta = entry.metadata()?;

            if meta.is_file() {
                database.handle_file(&cfg, &entry)?;
            }
        }

        debug!("{} {} done", "+".bright_black(), "+".bright_black(),);

        debug!("{} writing database", "+".bright_black());

        let mut buf = Vec::new();

        <Index as Message>::encode(&database.index, &mut buf)?;

        let mut encoder = GzEncoder::new(File::create(&database.index_path)?, Compression::best());

        io::copy(&mut &buf[..], &mut encoder)?;

        encoder.flush()?;

        Ok(database)
    }

    fn handle_file(&mut self, cfg: &Config, entry: &DirEntry) -> Result<()> {
        let path = entry.path();
        let ext = path.extension().and_then(OsStr::to_str);

        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

        match ext {
            Some("html") | Some("gz") => {
                let mut file = StoryFile::new(&path)?;

                let count = if cfg.trackers {
                    file.strip_trackers()?
                } else {
                    0
                };

                if count == 0 {
                    debug!(
                        "  {} found story: {}",
                        "|".bright_black(),
                        name.bright_green(),
                    );
                } else {
                    debug!(
                        "  {} found story: {} (with {} trackers, {})",
                        "|".bright_black(),
                        name.bright_green(),
                        count.bright_purple(),
                        "removed".bright_red(),
                    );
                }

                reader::read_story(self, name, &mut file.reader())
                    .with_context(|| name.to_owned())?;

                if cfg.compress {
                    file.compress()?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn get_chapter_body(&self, id: &str, chapter: usize) -> Result<String> {
        let story = self
            .index
            .stories
            .get(id)
            .ok_or_else(|| anyhow!("unable to find story in index"))?;

        let path = self.data_path.join(&story.file_name);

        let mut file = StoryFile::new(&path)?;

        let mut reader = file.reader();

        let mut contents = String::with_capacity(story.length.try_into()?);

        let _ = reader.read_to_string(&mut contents)?;

        let range = story.chapters.get(chapter - 1).ok_or_else(|| {
            anyhow!(
                "chapter `{}` not found, chapters: {}",
                chapter,
                story.chapters.len()
            )
        })?;

        Ok(contents
            .get((range.start.try_into()?)..(range.end.try_into()?))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "chapter `{}` not found in chapter index for `{}`",
                    chapter,
                    id
                )
            })?
            .to_owned())
    }

    #[allow(clippy::ptr_arg)]
    pub fn get_story_full<'i>(&self, id: &'i String) -> Result<(&'i String, StoryFull)> {
        enum Kind {
            Categories,
            Authors,
            Origins,
            Warnings,
            Pairings,
            Characters,
            Generals,
        }

        fn values(index: &Index, meta: &StoryMeta, kind: &Kind) -> Result<Vec<Entity>> {
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
                        .ok_or_else(|| anyhow!("entity with id `{}` does not exist", id))
                })
                .collect::<Result<Vec<_>>>()
        }

        let story_ref = self
            .index
            .stories
            .get(id)
            .ok_or_else(|| anyhow!("story with id `{}` does not exist", id))?;

        let index = &self.index;
        let meta = &story_ref.meta;

        Ok((
            id,
            StoryFull {
                file_name: story_ref.file_name.clone(),
                length: story_ref.length.try_into()?,
                chapters: story_ref
                    .chapters
                    .iter()
                    .map(Range::to_std)
                    .collect::<Result<Vec<_>>>()?,
                info: story_ref.info.clone(),
                meta: StoryFullMeta {
                    rating: Rating::from(story_ref.meta.rating),
                    categories: values(&index, &meta, &Kind::Categories).context("categories")?,
                    authors: values(&index, &meta, &Kind::Authors).context("authors")?,
                    origins: values(&index, &meta, &Kind::Origins).context("origins")?,
                    warnings: values(&index, &meta, &Kind::Warnings).context("warnings")?,
                    pairings: values(&index, &meta, &Kind::Pairings).context("pairings")?,
                    characters: values(&index, &meta, &Kind::Characters).context("characters")?,
                    generals: values(&index, &meta, &Kind::Generals).context("generals")?,
                },
            },
        ))
    }
}
