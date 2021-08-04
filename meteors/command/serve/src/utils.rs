use {
    common::{
        database::Database,
        models::{resolved, Entity, Existing, Index, StoryInfo, StoryMeta},
        prelude::*,
    },
    enrgy::HttpResponse,
    once_cell::sync::Lazy,
    std::{collections::BTreeMap, sync::RwLock},
};

pub fn wrap<F>(fun: F) -> HttpResponse
where
    F: FnOnce() -> Result<HttpResponse>,
{
    match fun() {
        Ok(res) => res,
        Err(err) => {
            error!("handler error: {}", err);

            HttpResponse::internal_server_error().finish()
        }
    }
}

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

static STORY_CACHE: Lazy<RwLock<BTreeMap<String, resolved::Story>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

#[allow(clippy::ptr_arg)]
pub fn get_story_full<'i>(db: &Database, id: &'i String) -> Result<(&'i String, resolved::Story)> {
    if let Some(story) = STORY_CACHE
        .read()
        .map_err(|err| anyhow!("unable to get lock on cache: {}", err))?
        .get(id)
        .cloned()
    {
        return Ok((id, story));
    }

    enum Kind {
        Categories,
        Authors,
        Origins,
        Warnings,
        Pairings,
        Characters,
        Generals,
    }

    fn values(index: &Index, meta: &StoryMeta, kind: &Kind) -> Result<Vec<Existing<Entity>>> {
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
                    .map(|entity| Existing::new(id.clone(), entity))
                    .ok_or_else(|| anyhow!("entity with id `{}` does not exist", id))
            })
            .collect::<Result<Vec<_>>>()
    }

    let index = db.index();

    let story_ref = index
        .stories
        .get(id)
        .ok_or_else(|| anyhow!("story with id `{}` does not exist", id))?;

    let info = &story_ref.info;
    let meta = &story_ref.meta;

    let story = resolved::Story {
        file_name: story_ref.file_name.clone(),
        file_hash: story_ref.file_hash,
        chapters: story_ref.chapters.clone(),
        info: StoryInfo {
            title: info.title.clone(),
            summary: info.summary.clone(),
        },
        meta: resolved::StoryMeta {
            rating: meta.rating,
            categories: values(index, meta, &Kind::Categories).context("categories")?,
            authors: values(index, meta, &Kind::Authors).context("authors")?,
            origins: values(index, meta, &Kind::Origins).context("origins")?,
            warnings: values(index, meta, &Kind::Warnings).context("warnings")?,
            pairings: values(index, meta, &Kind::Pairings).context("pairings")?,
            characters: values(index, meta, &Kind::Characters).context("characters")?,
            generals: values(index, meta, &Kind::Generals).context("generals")?,
        },
    };

    STORY_CACHE
        .write()
        .map_err(|err| anyhow!("unable to get lock on cache: {}", err))?
        .insert(id.clone(), story.clone());

    Ok((id, story))
}

pub struct Readable<N>
where
    N: std::fmt::Display,
{
    inner: N,
}

impl<N> std::fmt::Display for Readable<N>
where
    N: std::fmt::Display,
{
    #[allow(clippy::needless_collect)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values: Vec<(Option<char>, char)> = self
            .inner
            .to_string()
            .chars()
            .rev()
            .enumerate()
            .map(|(i, c)| {
                (
                    if i % 3 == 0 && i != 0 {
                        Some(',')
                    } else {
                        None
                    },
                    c,
                )
            })
            .collect();

        for (s, c) in values.into_iter().rev() {
            write!(f, "{}", c)?;

            if let Some(c) = s {
                write!(f, "{}", c)?;
            }
        }

        Ok(())
    }
}

pub trait IntoReadable: std::fmt::Display + Sized {
    fn into_readable(self) -> Readable<Self> {
        Readable { inner: self }
    }
}

impl IntoReadable for usize {}
impl IntoReadable for isize {}

impl IntoReadable for u32 {}
impl IntoReadable for u64 {}

impl IntoReadable for i32 {}
impl IntoReadable for i64 {}

pub struct Pagination {
    infix: Option<&'static str>,
    postfix: Option<&'static str>,
    pagers: Vec<Pager>,
    url: String,
}

impl Pagination {
    pub fn new(
        url: impl Into<String>,
        infix: Option<&'static str>,
        postfix: Option<&'static str>,
        pages: u32,
        page: u32,
    ) -> Self {
        Self {
            infix,
            postfix,
            pagers: Self::paginate(pages, page),
            url: url.into(),
        }
    }

    fn paginate(pages: u32, page: u32) -> Vec<Pager> {
        let mut buff = Vec::with_capacity(11);

        buff.push(Pager::Prev(page == 1, if page == 1 { 1 } else { page - 1 }));

        for i in 1..=pages {
            if i == 1 {
                buff.push(Pager::Num(i == page, i));

                continue;
            }

            if i == pages {
                buff.push(Pager::Num(i == page, i));

                continue;
            }

            if (page.checked_sub(1).unwrap_or(page)..=page.checked_add(1).unwrap_or(page))
                .contains(&i)
            {
                buff.push(Pager::Num(i == page, i));
            } else if let Some(l) = buff.last_mut() {
                if *l == Pager::Ellipse {
                    continue;
                } else {
                    buff.push(Pager::Ellipse);
                }
            }
        }

        buff.push(Pager::Next(
            page == pages,
            if page == pages { pages } else { page + 1 },
        ));

        buff
    }
}

impl std::fmt::Display for Pagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, r#"<div class="pagination">"#)?;

        let infix = self.infix.as_ref().unwrap_or(&"?page=");

        for pager in &self.pagers {
            match pager {
                Pager::Prev(d, n) => {
                    write!(f, r#"<li class="pagination__item"#)?;
                    if *d {
                        write!(f, " pagination__item--disabled")?;
                    }
                    write!(f, r#"">"#)?;

                    write!(f, r#"<a href="{}{}{}"#, self.url, infix, n)?;
                    if let Some(postfix) = self.postfix.as_ref() {
                        write!(f, "{}", postfix)?;
                    }
                    write!(f, r#"">prev</a>"#,)?;

                    writeln!(f, "</li>")?;
                }

                Pager::Num(d, n) => {
                    write!(
                        f,
                        r#"<li class="pagination__item{}"><a href="{}{}{}{}">"#,
                        if *d {
                            " pagination__item--disabled"
                        } else {
                            ""
                        },
                        self.url,
                        infix,
                        n,
                        if let Some(postfix) = self.postfix.as_ref() {
                            *postfix
                        } else {
                            ""
                        },
                    )?;

                    writeln!(f, "{}", n.into_readable())?;

                    writeln!(f, "</a></li>")?;
                }
                Pager::Ellipse => writeln!(f, r#"<li class="pagination__item"><p>...</p></li>"#)?,

                Pager::Next(d, n) => writeln!(
                    f,
                    r#"<li class="pagination__item{}"><a href="{}{}{}{}">next</a></li>"#,
                    if *d {
                        " pagination__item--disabled"
                    } else {
                        ""
                    },
                    self.url,
                    infix,
                    n,
                    if let Some(postfix) = self.postfix.as_ref() {
                        *postfix
                    } else {
                        ""
                    },
                )?,
            }
        }

        writeln!(f, "</div>")?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Pager {
    Prev(bool, u32),
    Num(bool, u32),
    Ellipse,
    Next(bool, u32),
}
