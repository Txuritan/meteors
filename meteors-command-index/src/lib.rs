use {
    common::{database::Database, models::proto::Index, prelude::*, Message},
    flate2::{write::GzEncoder, Compression},
    format_ao3::FileKind,
    std::{
        ffi::OsStr,
        fs::{self, DirEntry, File},
        io::{self, Read as _, Write as _},
    },
};

pub fn init() -> Result<Database> {
    debug!("{} building database", "+".bright_black());

    let mut database = Database::open()?;

    debug!(
        "{} {} checking data",
        "+".bright_black(),
        "+".bright_black(),
    );

    for entry in fs::read_dir(&database.data_path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_file() {
            handle_file(&mut database, &entry)?;
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

fn handle_file(db: &mut Database, entry: &DirEntry) -> Result<()> {
    let path = entry.path();
    let ext = path.extension().and_then(OsStr::to_str);

    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("File `{}` does not have a file name", path.display()))?;

    let pair = match ext {
        Some("html") => {
            let mut file = File::open(&path)?;

            debug!(
                "  {} found story: {}",
                "|".bright_black(),
                name.bright_green(),
            );

            Some((FileKind::Html, {
                let mut buf = String::new();

                file.read_to_string(&mut buf)?;

                buf
            }))
        }
        _ => None,
    };

    if let Some((kind, content)) = pair {
        format_ao3::parse(kind, &content).with_context(|| name.to_owned())?;
    }

    Ok(())
}
