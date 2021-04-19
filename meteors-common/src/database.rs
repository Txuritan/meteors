use {
    crate::{models::proto::Index, prelude::*},
    flate2::read::GzDecoder,
    prost::Message,
    std::{collections::BTreeMap, env, fs::File, io::Read as _, path::PathBuf},
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
    pub fn open() -> Result<Self> {
        let cur = env::current_dir()?.canonicalize()?;

        let data_path = cur.join("data");
        let index_path = cur.join("index.pb.gz");

        let database = if index_path.exists() {
            debug!("{} found existing", "+".bright_black());

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
            debug!("{} not found, creating", "+".bright_black());

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

        Ok(database)
    }
}
