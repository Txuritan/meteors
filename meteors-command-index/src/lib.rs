use {
    common::{
        database::Database,
        models::{Chapter, Entity, Story, StoryInfo, StoryMeta},
        prelude::*,
        utils::{self, FileIter},
        Action,
    },
    format_ao3::{FileKind, ParsedChapters, ParsedInfo, ParsedMeta},
    std::{
        collections::BTreeMap,
        ffi::OsStr,
        fs::{self, DirEntry},
        path::Path,
    },
};

#[derive(argh::FromArgs)]
#[argh(
    subcommand,
    name = "index",
    description = "builds or updates the index"
)]
pub struct Command {}

impl Action for Command {
    // open index
    // create id list
    // walk data dir
    //   get hash of file
    //   ignore if name and hash is in index, and id to list
    //   parse if not
    //   add parsed data to index removing old data, and id to list
    // remove id kv pairs that are not in the id list
    // write updated index
    #[allow(clippy::needless_collect)] // clippy doesn't detect that the keys are being removed
    fn run(&self) -> Result<()> {
        debug!("{} building index", "+".bright_black());

        let mut database = Database::open()?;

        let mut known_ids = Vec::new();

        debug!(
            "{} {} checking data",
            "+".bright_black(),
            "+".bright_black(),
        );

        for entry in FileIter::new(fs::read_dir(&database.data_path)?) {
            handle_entry(&mut database, &mut known_ids, entry?)?;
        }

        let index = database.index_mut();

        let index_keys = index.stories.keys().cloned().collect::<Vec<_>>();

        for id in index_keys
            .into_iter()
            .filter(|key| !known_ids.contains(&key))
        {
            match index.stories.remove(&id) {
                Some(story) => {
                    debug!(
                        "  {} removing missing story: {}",
                        "|".bright_black(),
                        story.file_name.bright_green(),
                    );
                }
                None => {
                    warn!(
                        "  {} removing nonexistent story with id `{}`",
                        "|".bright_black(),
                        id.bright_blue(),
                    );
                }
            }
        }

        debug!("{} {} done", "+".bright_black(), "+".bright_black());

        trace!(
            "{} found {} stories",
            "+".bright_black(),
            index.stories.len().bright_purple(),
        );

        database.save()?;

        Ok(())
    }
}

fn handle_entry(db: &mut Database, known_ids: &mut Vec<String>, entry: DirEntry) -> Result<()> {
    let path = entry.path();
    let ext = path.extension().and_then(OsStr::to_str);

    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

    let pair = match ext {
        Some("html") => Some(FileKind::Html),
        _ => None,
    };

    if let Some(kind) = pair {
        let data = handle_file(db, known_ids, &path, name)
            .with_context(|| format!("While reading file {}", name))?;

        if let Some((id, hash, bytes)) = data {
            let parsed = format_ao3::parse(kind, bytes).with_context(|| name.to_owned())?;

            add_to_index(db, name, hash, id, parsed);
        }
    }

    Ok(())
}

fn handle_file<P>(
    db: &mut Database,
    known_ids: &mut Vec<String>,
    path: P,
    name: &str,
) -> Result<Option<(String, u64, Vec<u8>)>>
where
    P: AsRef<Path>,
{
    let bytes = fs::read(&path)?;

    let hash = xxhash_rust::xxh3::xxh3_64(&bytes[..]);

    let index = db.index();

    let possible = index.stories.iter().find(|(_, v)| v.file_name == name);

    if let Some((id, story)) = possible {
        known_ids.push(id.clone());

        if story.file_hash == hash {
            // file hash is the same
            // it doesn't need to be updated
            Ok(None)
        } else {
            // file hash has changes
            // either it was overwritten with a new version
            // or it was edited in some way
            // either way the index entry needed to be updated
            debug!(
                "  {} found updated story: {}",
                "|".bright_black(),
                name.bright_green(),
            );

            Ok(Some((id.clone(), hash, bytes)))
        }
    } else {
        debug!(
            "  {} found new story: {}",
            "|".bright_black(),
            name.bright_green(),
        );

        let id = new_id(&index.stories);

        known_ids.push(id.clone());

        Ok(Some((id, hash, bytes)))
    }
}

fn add_to_index(
    db: &mut Database,
    name: &str,
    hash: u64,
    id: String,
    parsed: (ParsedInfo, ParsedMeta, ParsedChapters),
) {
    let (info, meta, chapters) = parsed;

    let index = db.index_mut();

    let story = Story {
        file_name: name.to_string(),
        file_hash: hash,
        chapters: chapters
            .chapters
            .into_iter()
            .map(|chapter| Chapter {
                title: chapter.title.to_string(),
                content: chapter.content,
                summary: chapter.summary,
                start_notes: chapter.start_notes,
                end_notes: chapter.end_notes,
            })
            .collect(),
        info: StoryInfo {
            title: info.title.to_string(),
            summary: info.summary.to_string(),
        },
        meta: StoryMeta {
            rating: meta.rating,
            authors: values_to_keys(info.authors, &mut index.authors),
            categories: values_to_keys(meta.categories, &mut index.categories),
            origins: values_to_keys(meta.origins, &mut index.origins),
            warnings: values_to_keys(meta.warnings, &mut index.warnings),
            pairings: values_to_keys(meta.pairings, &mut index.pairings),
            characters: values_to_keys(meta.characters, &mut index.characters),
            generals: values_to_keys(meta.generals, &mut index.generals),
        },
    };

    index.stories.insert(id, story);
}

fn values_to_keys(vec: Vec<String>, map: &mut BTreeMap<String, Entity>) -> Vec<String> {
    vec.into_iter()
        .map(|name| Database::get_default(map, name, new_id))
        .collect()
}

fn new_id<V>(map: &BTreeMap<String, V>) -> String {
    loop {
        let id = utils::new_id();

        if !map.contains_key(&id) {
            return id;
        }
    }
}
