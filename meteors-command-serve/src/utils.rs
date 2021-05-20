use common::{
    database::Database,
    models::{
        proto::{self, Entity, Index},
        story, Story,
    },
    prelude::*,
};

pub mod http {
    use {
        common::prelude::*,
        std::{fs, path::Path, process::Command},
    };

    pub fn get<P>(temp_path: P, url: &str) -> Result<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        let url = url.replace(
            &['\\', '/', ':', ';', '<', '>', '"', '|', '?', '*', '[', ']'][..],
            "-",
        );

        let temp_path = temp_path.as_ref();

        fs::create_dir_all(&temp_path)?;

        let temp_file_path = temp_path.join(&url);

        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let output = Command::new(shell)
            .args(&["/C", "curl"])
            .arg("-L")
            .arg("-o")
            .arg(&temp_file_path)
            .arg(&url)
            .output()?;

        if !output.status.success() {
            let mut err = anyhow!("curl return with error code {:?}", output.status);

            if let Ok(text) = String::from_utf8(output.stdout) {
                err = err.context(format!("with stdout: {}", text));
            }

            if let Ok(text) = String::from_utf8(output.stderr) {
                err = err.context(format!("with stderr: {}", text));
            }

            return Err(err);
        }

        let bytes = fs::read(&temp_file_path)?;

        fs::remove_file(&temp_file_path)?;

        Ok(bytes)
    }
}

#[allow(clippy::ptr_arg)]
pub fn get_story_full<'i>(db: &Database, id: &'i String) -> Result<(&'i String, Story)> {
    enum Kind {
        Categories,
        Authors,
        Origins,
        Warnings,
        Pairings,
        Characters,
        Generals,
    }

    fn values(index: &Index, meta: &proto::story::Meta, kind: &Kind) -> Result<Vec<Entity>> {
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

    let index = db.index();

    let story_ref = index
        .stories
        .get(id)
        .ok_or_else(|| anyhow!("story with id `{}` does not exist", id))?;

    let info = &story_ref.info();
    let meta = &story_ref.meta();

    Ok((
        id,
        Story {
            file_name: story_ref.file_name.clone(),
            file_hash: story_ref.file_hash,
            chapters: story_ref.chapters.clone(),
            info: story::Info {
                title: info.title.clone(),
                summary: info.summary.clone(),
            },
            meta: story::Meta {
                rating: meta.rating(),
                categories: values(&index, meta, &Kind::Categories).context("categories")?,
                authors: values(&index, meta, &Kind::Authors).context("authors")?,
                origins: values(&index, meta, &Kind::Origins).context("origins")?,
                warnings: values(&index, meta, &Kind::Warnings).context("warnings")?,
                pairings: values(&index, meta, &Kind::Pairings).context("pairings")?,
                characters: values(&index, meta, &Kind::Characters).context("characters")?,
                generals: values(&index, meta, &Kind::Generals).context("generals")?,
            },
        },
    ))
}
