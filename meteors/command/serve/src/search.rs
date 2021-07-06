use {
    common::{
        database::Database,
        models::{Entity, Index, Rating, Story},
    },
    std::{
        borrow::{Borrow as _, Cow},
        collections::{BTreeMap, HashMap},
        hash::Hash,
    },
};

pub fn search_v2<'s>(
    query: &[(Cow<'s, str>, Cow<'s, str>)],
    stories: &mut Vec<(&'s String, &'s Story)>,
) -> Stats<'s> {
    // modified version of [`Iterator::partition`] to remove [`Default`] bounds
    #[inline]
    fn partition<I, B, F>(iter: I, f: F) -> (Vec<B>, Vec<B>)
    where
        I: Iterator<Item = B>,
        F: FnMut(&I::Item) -> bool,
    {
        #[inline]
        fn extend<'a, T, B: Extend<T>>(
            mut f: impl FnMut(&T) -> bool + 'a,
            left: &'a mut B,
            right: &'a mut B,
        ) -> impl FnMut((), T) + 'a {
            move |(), x| {
                if f(&x) {
                    left.extend(Some(x));
                } else {
                    right.extend(Some(x));
                }
            }
        }

        let mut left: Vec<B> = Vec::new();
        let mut right: Vec<B> = Vec::new();

        iter.fold((), extend(f, &mut left, &mut right));

        (left, right)
    }

    #[inline]
    fn fn_filter<'i>((key, _value): &'i &(Cow<'_, str>, Cow<'_, str>)) -> bool {
        let include = Group::match_include(key.borrow());
        let exclude = Group::match_exclude(key.borrow());

        include | exclude
    }

    let (include, exclude) = partition(query.iter().filter(fn_filter), |(key, _value)| {
        Group::match_include(key.borrow())
    });

    let include = Group::from(include);
    let exclude = Group::from(exclude);

    include.filter(stories, true);
    exclude.filter(stories, false);

    Stats::new(&stories[..])
}

pub struct Stats<'m> {
    ratings: Vec<(Rating, usize)>,
    warnings: Vec<(&'m String, usize)>,
    categories: Vec<(&'m String, usize)>,
    origins: Vec<(&'m String, usize)>,
    pairings: Vec<(&'m String, usize)>,
    characters: Vec<(&'m String, usize)>,
    generals: Vec<(&'m String, usize)>,
}

pub struct StatKind {}

pub struct FilledStats<'m> {
    pub ratings: Vec<(Rating, usize)>,
    pub warnings: Vec<(&'m Entity, usize)>,
    pub categories: Vec<(&'m Entity, usize)>,
    pub origins: Vec<(&'m Entity, usize)>,
    pub pairings: Vec<(&'m Entity, usize)>,
    pub characters: Vec<(&'m Entity, usize)>,
    pub generals: Vec<(&'m Entity, usize)>,
}

impl<'m> Stats<'m> {
    fn new(stories: &[(&'m String, &'m Story)]) -> Stats<'m> {
        let mut ratings = HashMap::<Rating, usize>::new();

        let mut warnings = HashMap::<&String, usize>::new();
        let mut categories = HashMap::<&String, usize>::new();
        let mut origins = HashMap::<&String, usize>::new();
        let mut pairings = HashMap::<&String, usize>::new();
        let mut characters = HashMap::<&String, usize>::new();
        let mut generals = HashMap::<&String, usize>::new();

        fn inc<K>(map: &mut HashMap<K, usize>, entry: K)
        where
            K: Eq + Hash,
        {
            map.entry(entry)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        for (_, story) in stories {
            let meta = &story.meta;

            inc(&mut ratings, meta.rating);

            let lists = vec![
                (&mut warnings, &meta.warnings),
                (&mut categories, &meta.categories),
                (&mut origins, &meta.origins),
                (&mut pairings, &meta.pairings),
                (&mut characters, &meta.characters),
                (&mut generals, &meta.generals),
            ];

            for (map, list) in lists {
                for entry in list {
                    inc(map, entry);
                }
            }
        }

        fn to_top_list<K>(map: HashMap<K, usize>) -> Vec<(K, usize)>
        where
            K: Eq + Hash,
        {
            let mut list = map.into_iter().collect::<Vec<_>>();

            list.sort_by(|a, b| a.1.cmp(&b.1).reverse());

            list.truncate(10);

            list
        }

        Stats {
            ratings: to_top_list(ratings),
            warnings: to_top_list(warnings),
            categories: to_top_list(categories),
            origins: to_top_list(origins),
            pairings: to_top_list(pairings),
            characters: to_top_list(characters),
            generals: to_top_list(generals),
        }
    }

    pub fn fill(self, index: &'m Index) -> Option<FilledStats<'m>> {
        fn fill<'m>(
            list: Vec<(&'m String, usize)>,
            tree: &'m BTreeMap<String, Entity>,
        ) -> Option<Vec<(&'m Entity, usize)>> {
            list.into_iter()
                .map(|(id, count)| tree.get(id).map(|entity| (entity, count)))
                .collect::<Option<Vec<_>>>()
        }

        Some(FilledStats {
            ratings: self.ratings,
            warnings: fill(self.warnings, &index.warnings)?,
            categories: fill(self.categories, &index.categories)?,
            origins: fill(self.origins, &index.origins)?,
            pairings: fill(self.pairings, &index.pairings)?,
            characters: fill(self.characters, &index.characters)?,
            generals: fill(self.generals, &index.generals)?,
        })
    }
}

pub enum EntityKind {
    Origin,
    Pairing,
    Character,
    General,
}

#[derive(Default)]
struct Group<'i> {
    rating: Option<Vec<Cow<'i, str>>>,
    warnings: Option<Vec<Cow<'i, str>>>,
    categories: Option<Vec<Cow<'i, str>>>,
    origins: Option<Vec<Cow<'i, str>>>,
    characters: Option<Vec<Cow<'i, str>>>,
    pairings: Option<Vec<Cow<'i, str>>>,
    generals: Option<Vec<Cow<'i, str>>>,
}

impl<'i> Group<'i> {
    #[inline]
    fn match_include(text: &str) -> bool {
        matches!(text, "ir" | "iw" | "ict" | "io" | "ich" | "ip" | "ig")
    }

    #[inline]
    fn match_exclude(text: &str) -> bool {
        matches!(text, "er" | "ew" | "ect" | "eo" | "ech" | "ep" | "eg")
    }

    fn filter<'s>(self, stories: &mut Vec<(&'s String, &'s Story)>, include: bool) {
        let lists = vec![
            self.origins.map(|list| (EntityKind::Origin, list)),
            self.characters.map(|list| (EntityKind::Character, list)),
            self.pairings.map(|list| (EntityKind::Pairing, list)),
            self.generals.map(|list| (EntityKind::General, list)),
        ];

        for (kind, list) in lists.into_iter().flatten() {
            Self::filter_retain(stories, kind, include, list);
        }
    }

    fn filter_retain<'s>(
        stories: &mut Vec<(&'s String, &'s Story)>,
        kind: EntityKind,
        include: bool,
        entities: Vec<Cow<'i, str>>,
    ) {
        match kind {
            EntityKind::Origin => {
                for entity in entities {
                    stories.retain(|(_id, story)| {
                        include ^ story.meta.origins.iter().any(|id| id == &entity)
                    });
                }
            }
            EntityKind::Pairing => {
                for entity in entities {
                    stories.retain(|(_id, story)| {
                        include ^ story.meta.pairings.iter().any(|id| id == &entity)
                    });
                }
            }
            EntityKind::Character => {
                for entity in entities {
                    stories.retain(|(_id, story)| {
                        include ^ story.meta.characters.iter().any(|id| id == &entity)
                    });
                }
            }
            EntityKind::General => {
                for entity in entities {
                    stories.retain(|(_id, story)| {
                        include ^ story.meta.generals.iter().any(|id| id == &entity)
                    });
                }
            }
        }
    }
}

impl<'i> From<Vec<&(Cow<'i, str>, Cow<'i, str>)>> for Group<'i> {
    fn from(list: Vec<&(Cow<'i, str>, Cow<'i, str>)>) -> Self {
        let mut group: Group = Group::default();

        for (key, value) in list {
            let list = match key.borrow() {
                "ir" | "er" => Some(&mut group.rating),
                "iw" | "ew" => Some(&mut group.warnings),
                "ict" | "ect" => Some(&mut group.categories),
                "io" | "eo" => Some(&mut group.origins),
                "ich" | "ech" => Some(&mut group.characters),
                "ip" | "ep" => Some(&mut group.pairings),
                "ig" | "eg" => Some(&mut group.generals),
                _ => None,
            };

            if let Some(list) = list {
                list.get_or_insert_with(Vec::new).push(value.clone());
            }
        }

        group
    }
}

macro_rules! help {
    (bound; $db:ident, $iter:ident, $text:ident, $var:ident, $mem:ident) => {{
        BoundIter::$var(
            $iter.filter(move |(_, s)| any_by_text(&$db.index().$mem, &s.meta.$mem, &$text)),
        )
    }};
    (retain; $db:ident, $stories:ident, $include:ident, $text:ident, $mem:ident) => {{
        $stories.retain(|id| {
            let story = $db.index().stories.get(id).unwrap();

            !(&$include ^ any_by_text(&$db.index().$mem, &story.meta.$mem, &$text))
        });
    }};
}

#[derive(Debug, PartialEq)]
pub enum Bound {
    Author { include: bool, text: String },
    Origin { include: bool, text: String },
    Pairing { include: bool, text: String },
    Character { include: bool, text: String },
    General { include: bool, text: String },
}

impl Bound {
    const fn author(include: bool, text: String) -> Bound {
        Bound::Author { include, text }
    }

    const fn origin(include: bool, text: String) -> Bound {
        Bound::Origin { include, text }
    }

    const fn pairing(include: bool, text: String) -> Bound {
        Bound::Pairing { include, text }
    }

    const fn character(include: bool, text: String) -> Bound {
        Bound::Character { include, text }
    }

    const fn general(include: bool, text: String) -> Bound {
        Bound::General { include, text }
    }
}

pub fn search(database: &Database, text: &str) -> Vec<String> {
    let bounds = parse(text);

    let mut stories = Vec::new();

    let mut bounds_iter = bounds.into_iter();

    if let Some(bound) = bounds_iter.next() {
        let story_iter = database.index().stories.iter();

        let (include, iter) = match bound {
            Bound::Author { include, text } => (
                include,
                help!(bound; database, story_iter, text, Author, authors),
            ),
            Bound::Origin { include, text } => (
                include,
                help!(bound; database, story_iter, text, Origin, origins),
            ),
            Bound::Pairing { include, text } => (
                include,
                help!(bound; database, story_iter, text, Pairing, pairings),
            ),
            Bound::Character { include, text } => (
                include,
                help!(bound; database, story_iter, text, Character, characters),
            ),
            Bound::General { include, text } => (
                include,
                help!(bound; database, story_iter, text, General, generals),
            ),
        };

        first_push(include, database, &mut stories, iter);
    }

    for bound in bounds_iter {
        match bound {
            Bound::Author { include, text } => {
                help!(retain; database, stories, include, text, authors);
            }
            Bound::Origin { include, text } => {
                help!(retain; database, stories, include, text, origins);
            }
            Bound::Pairing { include, text } => {
                help!(retain; database, stories, include, text, pairings);
            }
            Bound::Character { include, text } => {
                help!(retain; database, stories, include, text, characters);
            }
            Bound::General { include, text } => {
                help!(retain; database, stories, include, text, generals);
            }
        }
    }

    stories
}

fn first_push<'d, I>(include: bool, database: &Database, stories: &mut Vec<String>, ids: I)
where
    I: Iterator<Item = (&'d String, &'d Story)>,
{
    if include {
        for id in ids.map(|(id, _)| id) {
            if !stories.contains(id) {
                stories.push(id.clone());
            }
        }
    } else {
        let ids = ids.map(|(id, _)| id).collect::<Vec<_>>();

        for id in database.index().stories.iter().map(|(id, _)| id) {
            if !ids.contains(&id) {
                stories.push(id.clone());
            }
        }
    }
}

fn any_by_text(full: &BTreeMap<String, Entity>, refs: &[String], text: &str) -> bool {
    refs.iter().map(|id| full.get(id)).any(|a| match a {
        Some(entity) => entity.text.to_lowercase() == text.to_lowercase(),
        None => false,
    })
}

#[allow(clippy::while_let_on_iterator)]
pub(self) fn parse(text: &str) -> Vec<Bound> {
    let cleaned = text.trim();
    let mut parts = cleaned.split(',').map(str::trim);

    let mut bounds = Vec::with_capacity(parts.size_hint().0);

    while let Some(mut part) = parts.next() {
        let included = !part.starts_with('-');

        if !included {
            part = part.trim_start_matches('-');
        }

        if parse_group(
            ["[", "]", "/"],
            &mut bounds,
            &mut parts,
            included,
            &mut part,
        ) {
            continue;
        }

        if parse_group(
            ["(", ")", " & "],
            &mut bounds,
            &mut parts,
            included,
            &mut part,
        ) {
            continue;
        }

        if parse_prefixed(
            ["a:", "author:"],
            Bound::author,
            &mut bounds,
            included,
            &mut part,
        ) {
            continue;
        }

        if parse_prefixed(
            ["o:", "origin:"],
            Bound::origin,
            &mut bounds,
            included,
            &mut part,
        ) {
            continue;
        }

        if parse_prefixed(
            ["c:", "character:"],
            Bound::character,
            &mut bounds,
            included,
            &mut part,
        ) {
            continue;
        }

        if parse_prefixed(
            ["g:", "general:"],
            Bound::general,
            &mut bounds,
            included,
            &mut part,
        ) {
            continue;
        }

        bounds.push(Bound::general(included, part.to_owned()));
    }

    bounds
}

fn parse_prefixed<B>(
    prefixes: [&str; 2],
    builder: B,
    bounds: &mut Vec<Bound>,
    included: bool,
    part: &mut &str,
) -> bool
where
    B: FnOnce(bool, String) -> Bound,
{
    let [short, long] = prefixes;

    if part.starts_with(short) || part.starts_with(long) {
        let part = part
            .trim_start_matches(short)
            .trim_start_matches(long)
            .to_owned();

        bounds.push(builder(included, part));

        true
    } else {
        false
    }
}

fn parse_group<'i, I>(
    symbols: [&str; 3],
    bounds: &mut Vec<Bound>,
    parts: &mut I,
    included: bool,
    part: &mut &str,
) -> bool
where
    I: Iterator<Item = &'i str>,
{
    let [open, close, sep] = symbols;

    if part.starts_with(open) {
        let mut part = part.trim_start_matches(open).to_owned();

        part.push_str(sep);

        for mut inner in parts {
            if inner.ends_with(close) {
                inner = inner.trim_end_matches(close);

                part.push_str(inner);

                break;
            }

            part.push_str(inner);
            part.push_str(sep);
        }

        bounds.push(Bound::pairing(included, part));

        true
    } else {
        false
    }
}

enum BoundIter<I, A, O, P, C, G>
where
    A: Iterator<Item = I>,
    O: Iterator<Item = I>,
    P: Iterator<Item = I>,
    C: Iterator<Item = I>,
    G: Iterator<Item = I>,
{
    Author(A),
    Origin(O),
    Pairing(P),
    Character(C),
    General(G),
}

impl<I, A, O, P, C, G> Iterator for BoundIter<I, A, O, P, C, G>
where
    A: Iterator<Item = I>,
    O: Iterator<Item = I>,
    P: Iterator<Item = I>,
    C: Iterator<Item = I>,
    G: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BoundIter::Author(i) => i.next(),
            BoundIter::Origin(i) => i.next(),
            BoundIter::Pairing(i) => i.next(),
            BoundIter::Character(i) => i.next(),
            BoundIter::General(i) => i.next(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($module:ident, $new:expr, [$prefix_long:expr, $prefix_short:expr]) => {
            mod $module {
                use super::*;

                const NEW: fn(bool, String) -> Bound = $new;

                test!(NEW, [$prefix_long, $prefix_short]);
            }
        };
        ($new:ident, [$prefix_short:expr, $prefix_long:expr]) => {
            #[test]
            fn test_prefix_long() {
                assert_eq!(
                    vec![
                        $new(true, "tag 1".to_owned()),
                        $new(true, "tag 2".to_owned()),
                    ],
                    parse(concat!($prefix_long, ":tag 1, ", $prefix_long, ":tag 2"))
                )
            }

            #[test]
            fn test_prefix_short() {
                assert_eq!(
                    vec![
                        $new(true, "tag 1".to_owned()),
                        $new(true, "tag 2".to_owned()),
                    ],
                    parse(concat!($prefix_short, ":tag 1, ", $prefix_short, ":tag 2"))
                )
            }

            #[test]
            fn test_exclude_prefix_long() {
                assert_eq!(
                    vec![
                        $new(false, "tag 1".to_owned()),
                        $new(false, "tag 2".to_owned()),
                    ],
                    parse(concat!(
                        "-",
                        $prefix_long,
                        ":tag 1, -",
                        $prefix_long,
                        ":tag 2"
                    ))
                )
            }

            #[test]
            fn test_exclude_prefix_short() {
                assert_eq!(
                    vec![
                        $new(false, "tag 1".to_owned()),
                        $new(false, "tag 2".to_owned()),
                    ],
                    parse(concat!(
                        "-",
                        $prefix_short,
                        ":tag 1, -",
                        $prefix_short,
                        ":tag 2"
                    ))
                )
            }
        };
    }

    test!(author, Bound::author, ["author", "a"]);

    test!(origin, Bound::origin, ["origin", "o"]);

    test!(character, Bound::character, ["character", "c"]);

    mod pairing {
        use super::*;

        const NEW: fn(bool, String) -> Bound = Bound::pairing;

        #[test]
        fn test_romantic() {
            assert_eq!(
                vec![NEW(true, "tag 1/tag 2".to_owned()),],
                parse("[tag 1, tag 2]")
            )
        }

        #[test]
        fn test_platonic() {
            assert_eq!(
                vec![NEW(true, "tag 1 & tag 2".to_owned()),],
                parse("(tag 1, tag 2)")
            )
        }

        #[test]
        fn test_exclude_romantic() {
            assert_eq!(
                vec![NEW(false, "tag 1/tag 2".to_owned()),],
                parse("-[tag 1, tag 2]")
            )
        }

        #[test]
        fn test_exclude_platonic() {
            assert_eq!(
                vec![NEW(false, "tag 1 & tag 2".to_owned()),],
                parse("-(tag 1, tag 2)")
            )
        }
    }

    mod general {
        use super::*;

        const NEW: fn(bool, String) -> Bound = Bound::general;

        #[test]
        fn test_no_prefix() {
            assert_eq!(
                vec![NEW(true, "tag 1".to_owned()), NEW(true, "tag 2".to_owned()),],
                parse("tag 1, tag 2")
            )
        }

        #[test]
        fn test_exclude_no_prefix() {
            assert_eq!(
                vec![
                    NEW(false, "tag 1".to_owned()),
                    NEW(false, "tag 2".to_owned()),
                ],
                parse("-tag 1, -tag 2")
            )
        }

        test!(NEW, ["general", "g"]);
    }
}
