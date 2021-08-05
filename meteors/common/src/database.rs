use {
    crate::{
        models::{Entity, Index, Meteors, Settings, Theme, Version},
        prelude::*,
        utils::FileIter,
    },
    aloene::Aloene as _,
    fs2::FileExt as _,
    memmap2::Mmap,
    std::{
        collections::BTreeMap,
        env,
        ffi::OsStr,
        fs::{self, File},
        mem,
        path::PathBuf,
    },
};

#[derive(Debug)]
pub struct Database {
    pub inner: Meteors,

    pub data_path: PathBuf,
    pub index_path: PathBuf,
    pub temp_path: PathBuf,

    lock_maps: BTreeMap<String, MappedFile>,
}

impl Database {
    pub fn open() -> Result<Self> {
        let cur = env::current_dir()?.canonicalize()?;

        let data_path = cur.join("data");
        let index_path = cur.join("meteors.aloe.dfl");
        let temp_path = cur.join("temp");

        let database = if index_path.exists() {
            debug!("found existing");

            // let mut decoder = GzDecoder::new(File::open(&index_path)?);

            // let mut bytes = Vec::new();

            // decoder.read_to_end(&mut bytes)?;

            let content = fs::read(&index_path)?;

            let bytes = miniz_oxide::inflate::decompress_to_vec(&content)
                .map_err(|err| anyhow::anyhow!("{:?}", err))?;

            let inner = Meteors::deserialize(&mut std::io::Cursor::new(bytes))?;

            Self {
                inner,

                data_path,
                index_path,
                temp_path,

                lock_maps: BTreeMap::new(),
            }
        } else {
            debug!("not found, creating");

            Self {
                inner: Meteors {
                    version: Version::V1,
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
                    settings: Settings {
                        theme: Theme::Light,
                        sync_key: String::new(),
                        nodes: vec![],
                    },
                },

                data_path,
                index_path,
                temp_path,

                lock_maps: BTreeMap::new(),
            }
        };

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

    pub fn lock_data(&mut self) -> Result<()> {
        let mut lock_maps = BTreeMap::new();

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
                .find(|(_, story)| story.file_name == name)
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
        let mut lock_maps = BTreeMap::new();

        mem::swap(&mut self.lock_maps, &mut lock_maps);

        for id in self.index().stories.keys().cloned() {
            if let Some(mapped) = lock_maps.remove(&id) {
                let MappedFile { name, file, map } = mapped;

                drop(map);

                file.unlock()
                    .with_context(|| format!("Unable to unlock `{}`", name))?;
            }
        }

        Ok(())
    }

    pub fn get_default<K>(map: &mut BTreeMap<String, Entity>, value: String, key: K) -> String
    where
        K: FnOnce(&BTreeMap<String, Entity>) -> String,
    {
        if let Some((key, _)) = map.iter().find(|(_, v)| v.text == value) {
            key.clone()
        } else {
            let key = key(&*map);

            map.insert(key.clone(), Entity { text: value });

            key
        }
    }

    pub fn get_chapter_body(&self, id: &str, number: usize) -> Result<String> {
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

    pub fn save(&self) -> Result<()> {
        debug!("writing index");

        let mut buf = Vec::new();

        Meteors::serialize(&self.inner, &mut buf)?;

        // let mut encoder = GzEncoder::new(File::create(&self.index_path)?, Compression::best());

        // io::copy(&mut &buf[..], &mut encoder)?;

        // encoder.flush()?;

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
