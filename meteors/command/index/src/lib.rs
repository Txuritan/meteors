use {
    common::{
        database::Database,
        models::{Chapter, Entity, FileKind, Site, Story, StoryInfo, StoryMeta},
        prelude::*,
        utils::{self, FileIter},
    },
    format_ao3::{ParsedChapters, ParsedInfo, ParsedMeta},
    std::{
        collections::BTreeMap,
        ffi::OsStr,
        fs::{self, DirEntry},
        hash::Hasher as _,
        io::{Cursor, Read as _},
        path::Path,
    },
    zip::{result::ZipError, ZipArchive},
};

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

    debug!("checking data",);

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
                    story.file_name.bright_green(),
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

enum FileKindReader {
    Epub(ZipArchive<Cursor<Vec<u8>>>),
    Html(String),
}

fn handle_entry(db: &mut Database, known_ids: &mut Vec<String>, entry: DirEntry) -> Result<()> {
    let path = entry.path();
    let ext = path.extension().and_then(OsStr::to_str);

    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

    let pair = match ext {
        Some("epub") => Some(FileKind::Epub),
        Some("html") => Some(FileKind::Html),
        _ => None,
    };

    if let Some(kind) = pair {
        let data = handle_file(db, known_ids, &path, name)
            .with_context(|| format!("While reading file {}", name))?;

        if let Some((id, hash, bytes)) = data {
            let (site, reader) = detect_site(kind, bytes)?;

            let parsed = match reader {
                FileKindReader::Epub(archive) => match site {
                    Site::ArchiveOfOurOwn => format_ao3::parse_epub(archive),
                    Site::Unknown => todo!(),
                },
                FileKindReader::Html(text) => match site {
                    Site::ArchiveOfOurOwn => format_ao3::parse_html(&text),
                    Site::Unknown => todo!(),
                },
            }
            .with_context(|| name.to_owned())?;

            add_to_index(db, name, hash, id, site, parsed);
        }
    }

    Ok(())
}

fn detect_site(kind: FileKind, bytes: Vec<u8>) -> Result<(Site, FileKindReader)> {
    struct Detector {
        content: &'static str,
        html: &'static str,
    }

    static DETECTOR_AO3: Detector = Detector {
        content: "<dc:publisher>Archive of Our Own</dc:publisher>",
        html: "Posted originally on the <a href=\"http://archiveofourown.org/\">Archive of Our Own</a>",
    };

    match kind {
        FileKind::Epub => {
            let mut archive = ZipArchive::new(Cursor::new(bytes))?;

            let site = match archive.by_name("content.opf") {
                Ok(mut file) => {
                    let mut text = String::with_capacity(file.size() as usize);

                    let _ = file.read_to_string(&mut text)?;

                    if text.contains(DETECTOR_AO3.content) {
                        Site::ArchiveOfOurOwn
                    } else {
                        Site::Unknown
                    }
                }
                Err(ZipError::FileNotFound) => Site::Unknown,
                Err(err) => return Err(err.into()),
            };

            Ok((site, FileKindReader::Epub(archive)))
        }
        FileKind::Html => {
            let text = String::from_utf8(bytes)?;

            let site = if text.contains(DETECTOR_AO3.html) {
                Site::ArchiveOfOurOwn
            } else {
                Site::Unknown
            };

            Ok((site, FileKindReader::Html(text)))
        }
    }
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

    let hash = {
        let mut hasher = crc32fast::Hasher::default();

        hasher.write(&bytes[..]);

        hasher.finish()
    };

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
            debug!("  found updated story: {}", name.bright_green(),);

            Ok(Some((id.clone(), hash, bytes)))
        }
    } else {
        debug!("  found new story: {}", name.bright_green(),);

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
    site: Site,
    parsed: (ParsedInfo, ParsedMeta, ParsedChapters),
) {
    let (info, meta, chapters) = parsed;

    let index = db.index_mut();

    let story = Story {
        file_name: name.to_string(),
        file_hash: hash,
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
