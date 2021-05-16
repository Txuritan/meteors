use common::{
    database::Database,
    models::{
        proto::{self, Entity, Index},
        story, Story,
    },
    prelude::*,
};

pub mod http {
    use {common::prelude::*, curl::easy::Easy, std::cell::RefCell};

    thread_local! {
        // cURL says to not use the same handle on multiple threads
        static HANDLE: RefCell<Easy> = RefCell::new(Easy::new());
    }

    pub fn get(url: &str) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        HANDLE.try_with(|handle| -> Result<()> {
            let mut handle = handle.borrow_mut();

            handle.fail_on_error(true)?;
            handle.follow_location(true)?;
            handle.useragent("meteors/1.0 (txuritan@github.com)")?;
            handle.url(url)?;

            let mut transfer = handle.transfer();

            transfer.write_function(|data| {
                buf.extend_from_slice(data);

                Ok(data.len())
            })?;

            transfer.perform()?;

            Ok(())
        })??;

        Ok(buf)
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
