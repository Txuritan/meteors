use {
    crate::{
        models::{Config, Entity, EntityKind, Id, Index, Settings, Theme, Version},
        prelude::*,
        utils::FileIter,
    },
    aloene::Aloene as _,
    fs2::FileExt as _,
    memmap2::Mmap,
    std::{
        collections::HashMap,
        env,
        ffi::OsStr,
        fs::{self, File},
        mem,
        path::PathBuf,
    },
};

#[derive(Debug)]
pub struct Database {
    inner: Config,

    pub data_path: PathBuf,
    pub index_path: PathBuf,
    pub temp_path: PathBuf,

    lock_maps: HashMap<Id, MappedFile>,
}

impl Database {
    pub fn open() -> Result<Self> {
        let cur = env::current_dir()?.canonicalize()?;

        let index_path = cur.join("varela.aloe.dfl");

        let database = if index_path.exists() {
            debug!("found existing");

            let content = fs::read(&index_path)?;

            let bytes = miniz_oxide::inflate::decompress_to_vec(&content)
                .map_err(|err| anyhow::anyhow!("unable to decompress index: {:?}", err))?;

            let inner =
                Config::deserialize(&mut &bytes[..]).context("unable to deserialize index")?;

            let data_path = PathBuf::from(&inner.settings.data_path);
            let temp_path = PathBuf::from(&inner.settings.temp_path);

            Self {
                inner,

                data_path,
                index_path,
                temp_path,

                lock_maps: HashMap::new(),
            }
        } else {
            debug!("not found, creating");

            let data_path = cur.join("data");
            let temp_path = cur.join("temp");

            Self {
                inner: Config {
                    version: Version::V1,
                    index: Index {
                        stories: HashMap::new(),
                        categories: HashMap::new(),
                        authors: HashMap::new(),
                        origins: HashMap::new(),
                        warnings: HashMap::new(),
                        pairings: HashMap::new(),
                        characters: HashMap::new(),
                        generals: HashMap::new(),
                    },
                    settings: Settings {
                        theme: Theme::Light,
                        sync_key: String::new(),
                        data_path: data_path
                            .to_str()
                            .ok_or_else(|| anyhow!("data path with not valid utf-8"))?
                            .to_string(),
                        temp_path: temp_path
                            .to_str()
                            .ok_or_else(|| anyhow!("temp path with not valid utf-8"))?
                            .to_string(),
                        nodes: vec![],
                    },
                },

                data_path,
                index_path,
                temp_path,

                lock_maps: HashMap::new(),
            }
        };

        fs::create_dir_all(&database.data_path)?;
        fs::create_dir_all(&database.temp_path)?;

        Ok(database)
    }

    pub fn index(&self) -> &Index {
        &self.inner.index
    }

    pub fn index_mut(&mut self) -> &mut Index {
        &mut self.inner.index
    }

    pub fn settings(&self) -> &Settings {
        &self.inner.settings
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.inner.settings
    }

    pub fn get_entity_from_id(&self, id: &Id) -> Option<EntityKind> {
        let index = self.index();

        let trees = [
            (&index.authors, EntityKind::Author),
            (&index.characters, EntityKind::Character),
            (&index.generals, EntityKind::General),
            (&index.origins, EntityKind::Origin),
            (&index.pairings, EntityKind::Pairing),
            (&index.warnings, EntityKind::Warning),
        ];

        for (tree, kind) in trees {
            if tree.contains_key(id) {
                return Some(kind);
            }
        }

        None
    }

    pub fn get_default<K>(map: &mut HashMap<Id, Entity>, value: String, key: K) -> Id
    where
        K: FnOnce(&HashMap<Id, Entity>) -> Id,
    {
        if let Some((key, _)) = map.iter().find(|(_, v)| v.text == value) {
            key.clone()
        } else {
            let key = key(&*map);

            map.insert(key.clone(), Entity { text: value });

            key
        }
    }

    pub fn get_chapter_body(&self, id: &Id, number: usize) -> Result<String> {
        let story = self
            .index()
            .stories
            .get(id)
            .ok_or_else(|| anyhow!("unable to find story in index"))?;

        if let Some(mapped) = self.lock_maps.get(id) {
            let contents = mapped.map.as_ref();

            let chapter = story.chapters.get(number - 1).ok_or_else(|| {
                anyhow!(
                    "chapter `{}` not found, chapters: {}",
                    number,
                    story.chapters.len()
                )
            })?;

            let sliced = contents.get(chapter.content.clone()).ok_or_else(|| {
                anyhow!(
                    "chapter `{}` not found in chapter index for `{}`",
                    number,
                    id,
                )
            })?;

            Ok(String::from_utf8(sliced.to_vec())?)
        } else {
            bail!("unable to find story in locked mapped index")
        }
    }

    pub fn lock_data(&mut self) -> Result<()> {
        let mut lock_maps = HashMap::new();

        for entry in FileIter::new(fs::read_dir(&self.data_path)?) {
            let entry = entry?;
            let path = entry.path();

            let name = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

            let id = self
                .index()
                .stories
                .iter()
                .find(|(_, story)| story.info.file_name == name)
                .map(|(id, _)| id.clone());

            if let Some(id) = id {
                let file = File::open(&path)?;

                file.try_lock_exclusive()
                    .with_context(|| format!("Unable to lock `{}`", name))?;

                let map = unsafe {
                    Mmap::map(&file).with_context(|| format!("Unable to memory map `{}`", name))?
                };

                lock_maps.insert(
                    id.clone(),
                    MappedFile {
                        name: name.to_string(),
                        file,
                        map,
                    },
                );
            }
        }

        mem::swap(&mut self.lock_maps, &mut lock_maps);

        Ok(())
    }

    pub fn unlock_data(&mut self) -> Result<()> {
        let mut lock_maps = HashMap::new();

        mem::swap(&mut self.lock_maps, &mut lock_maps);

        for id in self.index().stories.keys() {
            if let Some(mapped) = lock_maps.remove(id) {
                let MappedFile { name, file, map } = mapped;

                drop(map);

                file.unlock()
                    .with_context(|| format!("Unable to unlock `{}`", name))?;
            }
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        debug!("writing index");

        let mut buf = Vec::new();

        Config::serialize(&self.inner, &mut buf).context("unable to serialize index")?;

        let compressed = miniz_oxide::deflate::compress_to_vec(&buf, 10);

        fs::write(&self.index_path, &compressed)?;

        Ok(())
    }
}

#[derive(Debug)]
struct MappedFile {
    name: String,
    file: File,
    map: Mmap,
}
