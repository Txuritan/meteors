use std::io::Seek;

use {
    crate::{
        models::{Entity, Story, StoryMetaFull, StoryMetaRef},
        prelude::*,
        reader,
        regex::REGEX,
        Config,
    },
    flate2::{read::GzDecoder, write::GzEncoder, Compression},
    rand::{rngs::StdRng, Rng as _, SeedableRng as _},
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
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Id(String);

impl Id {
    pub fn from_str(id: &str) -> Self {
        Id(id.to_string())
    }

    pub const SIZE: usize = 8;

    const LEN: usize = 54;
    const MASK: usize = Self::LEN.next_power_of_two() - 1;
    const STEP: usize = 8 * Self::SIZE / 5;

    pub fn new_rand() -> Self {
        static ALPHABET: [char; Id::LEN] = [
            '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
            'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D',
            'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
            'X', 'Y', 'Z',
        ];

        let mut id = String::new();

        loop {
            let mut rng = StdRng::from_entropy();
            let mut bytes = [0u8; Self::STEP];

            rng.fill(&mut bytes[..]);

            for &byte in &bytes {
                let byte = byte as usize & Self::MASK;

                if ALPHABET.len() > byte {
                    id.push(ALPHABET[byte]);

                    if id.len() == Self::SIZE {
                        return Id(id);
                    }
                }
            }
        }
    }

    pub fn as_string(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Database {
    pub stories: BTreeMap<Id, Story<StoryMetaRef>>,

    pub categories: BTreeMap<Id, Entity>,

    pub authors: BTreeMap<Id, Entity>,

    pub origins: BTreeMap<Id, Entity>,

    pub warnings: BTreeMap<Id, Entity>,
    pub pairings: BTreeMap<Id, Entity>,
    pub characters: BTreeMap<Id, Entity>,
    pub generals: BTreeMap<Id, Entity>,

    #[serde(skip)]
    pub data: PathBuf,

    #[serde(skip)]
    pub index: PathBuf,
}

impl Database {
    pub fn init(cfg: Config) -> Result<Self> {
        debug!("{} building database", "+".bright_black());

        let cur = env::current_dir()?.canonicalize()?;

        let data_path = cur.join("data");
        let index_path = cur.join("index.bc");

        let mut database = if index_path.exists() {
            debug!("{} found existing", "|".bright_black());

            let file = File::open(&index_path)?;

            let mut database: Database = bincode::deserialize_from(file)?;

            database.data = data_path.clone();
            database.index = index_path.clone();

            database
        } else {
            debug!("{} not found, creating", "|".bright_black());

            Self {
                stories: BTreeMap::new(),

                categories: BTreeMap::new(),

                authors: BTreeMap::new(),

                origins: BTreeMap::new(),

                warnings: BTreeMap::new(),
                pairings: BTreeMap::new(),
                characters: BTreeMap::new(),
                generals: BTreeMap::new(),

                data: data_path.clone(),
                index: index_path.clone(),
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

        let file = File::create(&index_path)?;

        debug!("{} writing database", "+".bright_black());

        bincode::serialize_into(file, &database)?;

        Ok(database)
    }

    fn handle_file(&mut self, cfg: &Config, entry: DirEntry) -> Result<()> {
        let path = entry.path();
        let ext = path.extension().and_then(OsStr::to_str);

        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| eyre!("File `{}` does not have a file name", path.display()))?;

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

    pub fn get_chapter_body(&self, id: &Id, chapter: usize) -> Result<String> {
        let story = self
            .stories
            .get(id)
            .ok_or_else(|| eyre!("unable to find story in index"))?;

        let path = self.data.join(&story.file_name);

        let mut reader = GzDecoder::new(BufReader::new(File::open(path)?));

        let mut contents = String::with_capacity(story.length);

        let _ = reader.read_to_string(&mut contents)?;

        let range = story.chapters.get(chapter - 1).ok_or_else(|| {
            eyre!(
                "chapter `{}` not found, chapters: {}",
                chapter,
                story.chapters.len()
            )
        })?;

        Ok(contents[range.clone()].to_string())
    }

    pub fn get_story_full<'i>(&self, id: &'i Id) -> Result<(&'i Id, Story<StoryMetaFull>)> {
        let story_ref = self
            .stories
            .get(id)
            .ok_or_else(|| eyre!("story with id `{}` does not exist", id))?;

        Ok((
            id,
            Story {
                file_name: story_ref.file_name.clone(),
                length: story_ref.length,
                chapters: story_ref.chapters.clone(),
                info: story_ref.info.clone(),
                meta: StoryMetaFull {
                    rating: story_ref.meta.rating,
                    categories: self
                        .get_all_values(&self.categories, &story_ref.meta.categories)
                        .context("categories")?,
                    authors: self
                        .get_all_values(&self.authors, &story_ref.meta.authors)
                        .context("authors")?,
                    origins: self
                        .get_all_values(&self.origins, &story_ref.meta.origins)
                        .context("origins")?,
                    warnings: self
                        .get_all_values(&self.warnings, &story_ref.meta.warnings)
                        .context("warnings")?,
                    pairings: self
                        .get_all_values(&self.pairings, &story_ref.meta.pairings)
                        .context("pairings")?,
                    characters: self
                        .get_all_values(&self.characters, &story_ref.meta.characters)
                        .context("characters")?,
                    generals: self
                        .get_all_values(&self.generals, &story_ref.meta.generals)
                        .context("generals")?,
                },
            },
        ))
    }

    fn get_all_values(&self, map: &BTreeMap<Id, Entity>, keys: &[Id]) -> Result<Vec<Entity>> {
        keys.iter()
            .map(|id| {
                map.get(id)
                    .cloned()
                    .ok_or_else(|| eyre!("entity with id `{}` does not exist", id))
            })
            .collect::<Result<Vec<_>>>()
    }
}
