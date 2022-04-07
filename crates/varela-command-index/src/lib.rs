use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, DirEntry},
    hash::Hasher as _,
    path::Path,
    time::SystemTime,
};

use common::{
    database::Database,
    models::{Chapter, Entity, FileKind, Id, Site, Story, StoryInfo, StoryMeta},
    prelude::*,
    utils::{self, FileIter},
};
use format_ao3::{ParsedChapters, ParsedInfo, ParsedMeta};

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
#[inline(never)]
pub fn run(_args: common::Args) -> Result<()> {
    debug!("building index");

    let mut database = Database::open()?;

    let mut known_ids = Vec::new();

    debug!(
        "checking `{}` for files",
        database.data_path.display().bright_purple()
    );

    for entry in FileIter::new(fs::read_dir(&database.data_path)?) {
        handle_entry(&mut database, &mut known_ids, entry?)?;
    }

    let index = database.index_mut();

    let index_keys = index.stories.keys().cloned().collect::<Vec<_>>();

    for id in index_keys
        .into_iter()
        .filter(|key| !known_ids.contains(key))
    {
        match index.stories.remove(&id) {
            Some(story) => {
                debug!(
                    "  removing missing story: {}",
                    story.info.file_name.bright_green(),
                );
            }
            None => {
                warn!(
                    "  removing nonexistent story with id `{}`",
                    id.bright_blue(),
                );
            }
        }
    }

    debug!("done");

    trace!("found {} stories", index.stories.len().bright_purple(),);

    database.save()?;

    Ok(())
}

fn handle_entry(db: &mut Database, known_ids: &mut Vec<Id>, entry: DirEntry) -> Result<()> {
    let path = entry.path();
    let ext = path.extension().and_then(OsStr::to_str);

    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

    let file_kind = match ext {
        // Some("epub") => Some(FileKind::Epub),
        Some("epub") => return Ok(()),
        Some("html") => Some(FileKind::Html),
        _ => None,
    };

    if let Some(kind) = file_kind {
        let details = handle_file(db, known_ids, &path, name)
            .with_context(|| format!("While reading file {}", name))?;

        if let Some((id, hash, updating)) = details {
            struct Detector {
                content: &'static str,
                html: &'static str,
            }

            static DETECTOR_AO3: Detector = Detector {
                content: "<dc:publisher>Archive of Our Own</dc:publisher>",
                html: r#"Posted originally on the <a href="http://archiveofourown.org/">Archive of Our Own</a>"#,
            };

            let temp_file_path = db.temp_path.join(&name.trim_end_matches(".epub"));

            let site = match kind {
                FileKind::Epub => {
                    let output = common::utils::command("unzip")
                        .arg(&path)
                        .arg("-d")
                        .arg(&temp_file_path)
                        .output()?;

                    match output.status.code() {
                        Some(0) | Some(1) => {
                            let content_opf = temp_file_path.join("content.opf");
                            let text = fs::read_to_string(&content_opf)?;

                            if text.contains(DETECTOR_AO3.content) {
                                Site::ArchiveOfOurOwn
                            } else {
                                Site::Unknown
                            }
                        }
                        Some(code) => {
                            anyhow::bail!("unable to decompress epub, unzip status code: {}", code);
                        }
                        None => {
                            anyhow::bail!("unable to decompress epub, no status code");
                        }
                    }
                }
                FileKind::Html => {
                    let text = fs::read_to_string(&path)?;

                    if text.contains(DETECTOR_AO3.html) {
                        Site::ArchiveOfOurOwn
                    } else {
                        Site::Unknown
                    }
                }
            };

            let parsed = match site {
                Site::ArchiveOfOurOwn => match kind {
                    FileKind::Epub => {
                        let parsed = format_ao3::parse_epub(&temp_file_path);

                        fs::remove_dir_all(&temp_file_path)?;

                        parsed?
                    }
                    FileKind::Html => {
                        let text = fs::read_to_string(&path)?;

                        format_ao3::parse_html(&text)?
                    }
                },
                Site::Unknown => {
                    return Ok(());
                }
            };

            add_to_index(db, name, hash, updating, id, kind, site, parsed);
        }
    }

    Ok(())
}

fn handle_file<P>(
    db: &mut Database,
    known_ids: &mut Vec<Id>,
    path: P,
    name: &str,
) -> Result<Option<(Id, u64, bool)>>
where
    P: AsRef<Path>,
{
    let bytes = fs::read(&path)?;

    let hash = {
        let mut hasher = crc32fast::Hasher::default();

        hasher.write(&bytes[..]);

        hasher.finish()
    };

    let index = db.index();

    let possible = index.stories.iter().find(|(_, v)| v.info.file_name == name);

    if let Some((id, story)) = possible {
        known_ids.push(id.clone());

        if story.info.file_hash == hash {
            // file hash is the same
            // it doesn't need to be updated
            Ok(None)
        } else {
            // file hash has changes
            // either it was overwritten with a new version
            // or it was edited in some way
            // either way the index entry needed to be updated
            debug!("  found updated story: {}", name.bright_green(),);

            Ok(Some((id.clone(), hash, true)))
        }
    } else {
        debug!("  found new story: {}", name.bright_green(),);

        let id = new_id(&index.stories);

        known_ids.push(id.clone());

        Ok(Some((id, hash, false)))
    }
}

#[allow(clippy::too_many_arguments)]
fn add_to_index(
    db: &mut Database,
    name: &str,
    hash: u64,
    updating: bool,
    id: Id,
    kind: FileKind,
    site: Site,
    parsed: (ParsedInfo, ParsedMeta, ParsedChapters),
) {
    let (info, meta, chapters) = parsed;

    let created = if updating {
        if let Some(created) = db.index().stories.get(&id).map(|story| &story.info.created) {
            created.clone()
        } else {
            humantime::format_rfc3339(SystemTime::now()).to_string()
        }
    } else {
        humantime::format_rfc3339(SystemTime::now()).to_string()
    };

    let index = db.index_mut();

    let story = Story {
        site,
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
            file_name: name.to_string(),
            file_hash: hash,
            title: info.title.to_string(),
            kind,
            summary: info.summary.to_string(),
            created,
            updated: humantime::format_rfc3339(SystemTime::now()).to_string(),
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

fn values_to_keys(vec: Vec<String>, map: &mut HashMap<Id, Entity>) -> Vec<Id> {
    vec.into_iter()
        .map(|name| Database::get_default(map, name, new_id))
        .collect()
}

fn new_id<V>(map: &HashMap<Id, V>) -> Id {
    loop {
        let id = utils::new_id();

        if !map.contains_key(&id) {
            return id;
        }
    }
}
