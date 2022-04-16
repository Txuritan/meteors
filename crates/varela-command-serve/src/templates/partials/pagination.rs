use crate::utils::IntoReadable as _;

#[derive(opal::Template)]
#[template(path = "partials/pagination.hbs")]
pub struct Pagination {
    prev: Link,
    parts: Vec<Link>,
    next: Link,
}

impl Pagination {
    pub fn new(url: String, page: u32, pages: u32) -> Self {
        let (prev, parts, next) = Self::paginate(&url, pages, page);

        Self { prev, parts, next }
    }

    fn paginate(url: &str, pages: u32, page: u32) -> (Link, Vec<Link>, Link) {
        let mut buff = Vec::with_capacity(11);

        let prev = Link {
            state: if page == 1 {
                LinkState::Disabled
            } else {
                LinkState::Normal
            },
            href: vfmt::format!("{}/{}", url, page),
            text: "previous".into(),
        };

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

        let next = Link {
            state: if page == pages {
                LinkState::Disabled
            } else {
                LinkState::Normal
            },
            href: vfmt::format!("{}/{}", url, page),
            text: "next".into(),
        };

        let buff = buff
            .into_iter()
            .map(|pager| match pager {
                Pager::Num(active, page) => Link {
                    state: if active {
                        LinkState::Active
                    } else {
                        LinkState::Normal
                    },
                    href: vfmt::format!("{}/{}", url, page),
                    text: page.into_readable().to_string(),
                },
                Pager::Ellipse => Link {
                    state: LinkState::Normal,
                    href: "#".into(),
                    text: "..".into(),
                },
            })
            .collect::<Vec<_>>();

        (prev, buff, next)
    }
}

#[derive(Debug, PartialEq)]
enum Pager {
    Num(bool, u32),
    Ellipse,
}

#[derive(opal::Template)]
#[template(path = "partials/pagination-link.hbs")]
struct Link {
    state: LinkState,
    href: String,
    text: String,
}

#[derive(Debug, PartialEq)]
enum LinkState {
    Normal,
    Active,
    Disabled,
}
