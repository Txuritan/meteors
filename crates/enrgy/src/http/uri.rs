//! A modified version of [uhttp_uri](https://github.com/kchmck/uhttp_uri.rs/tree/15bec5c1a4049cc7da297130e5c60e404a48cb2f).

use crate::utils::Const;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct HttpResource {
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl HttpResource {
    pub fn new(src: &str) -> Self {
        let (path, query, fragment) = Self::parts(src, src.find('?'), src.find('#'));

        HttpResource {
            path: if path.is_empty() {
                "/".to_string()
            } else {
                path.to_string()
            },
            query: if query.is_empty() {
                None
            } else {
                Some(query.to_string())
            },
            fragment: if fragment.is_empty() {
                None
            } else {
                Some(fragment.to_string())
            },
        }
    }

    // TODO(txuritan): all the `&s[..]` are blocking this from being const
    fn parts(
        s: &str,
        query_index: Option<usize>,
        fragment_index: Option<usize>,
    ) -> (&str, &str, &str) {
        match (query_index, fragment_index) {
            (Some(q), Some(f)) => {
                if q < f {
                    let (path, query) = Const::str_split_at(&s[..f], q);
                    let (_, frag) = Const::str_split_at(s, f);

                    (path, &query[1..], &frag[1..])
                } else {
                    Self::parts(s, None, Some(f))
                }
            }
            (Some(q), None) => {
                let (path, query) = Const::str_split_at(s, q);

                (path, &query[1..], "")
            }
            (None, Some(f)) => {
                let (path, frag) = Const::str_split_at(s, f);

                (path, "", &frag[1..])
            }
            (None, None) => (s, "", ""),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_http_resource() {
        assert_eq!(
            HttpResource::new("/a/b/c"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: None,
                fragment: None,
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c?key=val"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: Some("key=val".to_string()),
                fragment: None,
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c#frag"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: None,
                fragment: Some("frag".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c#frag?frag-param"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: None,
                fragment: Some("frag?frag-param".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c?key=val&param#frag"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: Some("key=val&param".to_string()),
                fragment: Some("frag".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c/?key=val&param#frag"),
            HttpResource {
                path: "/a/b/c/".to_string(),
                query: Some("key=val&param".to_string()),
                fragment: Some("frag".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c?key=d/e#frag/ment?param"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: Some("key=d/e".to_string()),
                fragment: Some("frag/ment?param".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/a/b/c#frag?param&key=val"),
            HttpResource {
                path: "/a/b/c".to_string(),
                query: None,
                fragment: Some("frag?param&key=val".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/%02/%03/%04#frag"),
            HttpResource {
                path: "/%02/%03/%04".to_string(),
                query: None,
                fragment: Some("frag".to_string()),
            }
        );

        assert_eq!(
            HttpResource::new("/"),
            HttpResource {
                path: "/".to_string(),
                query: None,
                fragment: None,
            }
        );

        assert_eq!(
            HttpResource::new(""),
            HttpResource {
                path: "/".to_string(),
                query: None,
                fragment: None,
            }
        );

        assert_eq!(
            HttpResource::new("?#"),
            HttpResource {
                path: "/".to_string(),
                query: None,
                fragment: None,
            }
        );

        assert_eq!(
            HttpResource::new("?key=val#"),
            HttpResource {
                path: "/".to_string(),
                query: Some("key=val".to_string()),
                fragment: None,
            }
        );

        assert_eq!(
            HttpResource::new("?#frag"),
            HttpResource {
                path: "/".to_string(),
                query: None,
                fragment: Some("frag".to_string()),
            }
        );
    }
}
