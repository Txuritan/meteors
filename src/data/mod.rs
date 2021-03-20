mod reader;
mod search;

use std::io::Seek;

use {
    crate::{
        models::{
            proto::{Entity, Index, Range, Rating, StoryMeta},
            StoryFull, StoryFullMeta,
        },
        prelude::*,
        regex::REGEX,
        Config,
    },
    flate2::{read::GzDecoder, write::GzEncoder, Compression},
    prost::Message,
    std::{
        collections::BTreeMap,
        env,
        ffi::OsStr,
        fs::{self, DirEntry, File},
        io::{self, BufReader, BufWriter, Read, SeekFrom, Write as _},
        path::{Path, PathBuf},
    },
};

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq)]
pub struct Database {
    pub index: Index,

    pub data_path: PathBuf,

    pub index_path: PathBuf,
}

impl Database {
    pub fn init(cfg: Config) -> Result<Self> {
        debug!("{} building database", "+".bright_black());

        let cur = env::current_dir()?.canonicalize()?;

        let data_path = cur.join("data");
        let index_path = cur.join("index.pb");

        let mut database = if index_path.exists() {
            debug!("{} found existing", "|".bright_black());

            let bytes = fs::read(&index_path)?;

            let index = <Index as Message>::decode(&bytes[..])?;

            Self {
                index,

                data_path: data_path.clone(),
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

                data_path: data_path.clone(),
                index_path,
            }
        };

        debug!(
            "{} {} checking data",
            "+".bright_black(),
            "+".bright_black(),
        );

        for entry in fs::read_dir(&data_path)? {
            let entry = entry?;
            let meta = entry.metadata()?;

            if meta.is_file() {
                database.handle_file(&cfg, entry)?;
            }
        }

        debug!("{} {} done", "+".bright_black(), "+".bright_black(),);

        debug!("{} writing database", "+".bright_black());

        let mut buf = Vec::new();

        <Index as Message>::encode(&database.index, &mut buf)?;

        fs::write(&database.index_path, &buf)?;

        Ok(database)
    }

    fn handle_file(&mut self, cfg: &Config, entry: DirEntry) -> Result<()> {
        let path = entry.path();
        let ext = path.extension().and_then(OsStr::to_str);

        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

        match ext {
            Some("html") => {
                let count = if cfg.trackers {
                    self.rewrite_story(&path)?
                } else {
                    0
                };

                if count != 0 {
                    debug!(
                        "  {} found story: {} (with {} trackers, {})",
                        "|".bright_black(),
                        name.bright_green(),
                        count.bright_purple(),
                        "removed".bright_red(),
                    );
                } else {
                    debug!(
                        "  {} found story: {}",
                        "|".bright_black(),
                        name.bright_green(),
                    );
                }

                let mut reader = BufReader::new(File::open(&path)?);

                reader::read_story(self, name, &mut reader).with_context(|| name.to_string())?;

                reader.seek(SeekFrom::Start(0))?;

                if cfg.compress {
                    self.compress_story(&path, &mut reader)?;

                    fs::remove_file(&path)?;
                }
            }
            Some("gz") => {
                debug!(
                    "  {} found story: {}",
                    "|".bright_black(),
                    name.bright_green(),
                );

                let mut reader = GzDecoder::new(BufReader::new(File::open(&path)?));

                reader::read_story(self, name, &mut reader).with_context(|| name.to_string())?;
            }
            _ => {}
        }

        Ok(())
    }

    fn compress_story<P, R>(&mut self, path: P, reader: &mut R) -> Result<()>
    where
        P: AsRef<Path>,
        R: Read,
    {
        let path = path.as_ref();

        let new_path = path.with_extension("html.gz");

        let mut writer = GzEncoder::new(File::create(&new_path)?, Compression::best());

        io::copy(reader, &mut writer)?;

        writer.flush()?;

        Ok(())
    }

    fn rewrite_story<P>(&mut self, path: P) -> Result<usize>
    where
        P: AsRef<Path>,
    {
        let mut reader = BufReader::new(File::open(&path)?);

        let mut in_buf = String::new();

        let _ = reader.read_to_string(&mut in_buf)?;

        let positions = REGEX
            .find_iter(in_buf.as_bytes())
            .map(|(start, end)| start..end)
            .collect::<Vec<_>>();

        let count = positions.len();

        if count != 0 {
            let mut out_buf = BufWriter::new(File::create(&path)?);

            for (i, byte) in in_buf.as_bytes().iter().enumerate() {
                if positions.iter().find(|range| range.contains(&i)).is_none() {
                    let _ = out_buf.write(&[*byte])?;
                }
            }

            out_buf.flush()?;
        }

        Ok(count)
    }

    pub fn get_chapter_body(&self, id: &str, chapter: usize) -> Result<String> {
        let story = self
            .index
            .stories
            .get(id)
            .ok_or_else(|| anyhow!("unable to find story in index"))?;

        let path = self.data_path.join(&story.file_name);

        let mut reader = GzDecoder::new(BufReader::new(File::open(path)?));

        let mut contents = String::with_capacity(story.length as usize);

        let _ = reader.read_to_string(&mut contents)?;

        let range = story.chapters.get(chapter - 1).ok_or_else(|| {
            anyhow!(
                "chapter `{}` not found, chapters: {}",
                chapter,
                story.chapters.len()
            )
        })?;

        Ok(contents[(range.start as usize)..(range.end as usize)].to_string())
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

        fn values(index: &Index, meta: &StoryMeta, kind: Kind) -> Result<Vec<Entity>> {
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
                length: story_ref.length as usize,
                chapters: story_ref
                    .chapters
                    .iter()
                    .map(Range::to_std)
                    .collect::<Vec<_>>(),
                info: story_ref.info.clone(),
                meta: StoryFullMeta {
                    rating: Rating::from(story_ref.meta.rating),
                    categories: values(&index, &meta, Kind::Categories).context("categories")?,
                    authors: values(&index, &meta, Kind::Authors).context("authors")?,
                    origins: values(&index, &meta, Kind::Origins).context("origins")?,
                    warnings: values(&index, &meta, Kind::Warnings).context("warnings")?,
                    pairings: values(&index, &meta, Kind::Pairings).context("pairings")?,
                    characters: values(&index, &meta, Kind::Characters).context("characters")?,
                    generals: values(&index, &meta, Kind::Generals).context("generals")?,
                },
            },
        ))
    }
}
