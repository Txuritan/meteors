use {
    crate::{
        database::{Database, Id},
        models::{Entity, Story, StoryMetaRef},
    },
    std::collections::BTreeMap,
};

macro_rules! help {
    (bound; $db:ident, $iter:ident, $text:ident, $var:ident, $mem:ident) => {{
        BoundIter::$var($iter.filter(move |(_, s)| any_by_text(&$db.$mem, &s.meta.$mem, &$text)))
    }};
    (retain; $db:ident, $stories:ident, $include:ident, $text:ident, $mem:ident) => {{
        $stories.retain(|id| {
            let story = $db.stories.get(id).unwrap();

            !(&$include ^ any_by_text(&$db.$mem, &story.meta.$mem, &$text))
        });
    }};
}

pub enum Bound {
    Author { include: bool, text: String },
    Origin { include: bool, text: String },
    Pairing { include: bool, text: String },
    Character { include: bool, text: String },
    General { include: bool, text: String },
}

#[allow(dead_code)]
#[allow(clippy::while_let_on_iterator)]
pub fn parse(text: &str) -> Vec<Bound> {
    let mut parts = text.split(',').map(|part| part.trim());

    let mut bounds = Vec::with_capacity(parts.size_hint().0);

    while let Some(mut part) = parts.next() {
        let included = part.starts_with('-');

        if included {
            part = part.trim_start_matches('-');
        }

        if part.starts_with('[') {
            let mut part = part.trim_start_matches('[').to_string();

            part.push('/');

            's_inner: while let Some(mut inner) = parts.next() {
                if inner.ends_with(']') {
                    inner = inner.trim_end_matches(']');

                    part.push_str(inner);

                    break 's_inner;
                }

                part.push_str(inner);
                part.push('/');
            }

            bounds.push(Bound::Pairing { include: included, text: part.to_owned() });

            continue;
        }

        if part.starts_with('(') {
            let mut part = part.trim_start_matches('(').to_string();

            part.push_str(" & ");

            'p_inner: while let Some(mut inner) = parts.next() {
                if inner.ends_with(')') {
                    inner = inner.trim_end_matches(')');

                    part.push_str(inner);

                    break 'p_inner;
                }

                part.push_str(inner);
                part.push_str(" & ");
            }

            bounds.push(Bound::Pairing { include: included, text: part.to_owned() });

            continue;
        }

        if part.starts_with("a:") || part.starts_with("author:") {
            let part = part.trim_start_matches("a:").trim_start_matches("author:").to_string();

            bounds.push(Bound::Author { include: included, text: part.to_owned() });

            continue;
        }

        if part.starts_with("o:") || part.starts_with("origin:") {
            let part = part.trim_start_matches("o:").trim_start_matches("origin:").to_string();

            bounds.push(Bound::Origin { include: included, text: part.to_owned() });

            continue;
        }

        if part.starts_with("c:") || part.starts_with("character:") {
            let part = part.trim_start_matches("c:").trim_start_matches("character:").to_string();

            bounds.push(Bound::Character { include: included, text: part.to_owned() });

            continue;
        }

        bounds.push(Bound::General { include: included, text: part.to_owned() });
    }

    bounds
}

#[allow(dead_code)]
pub fn search(database: &Database, bounds: Vec<Bound>) -> Vec<Id> {
    let mut stories = Vec::new();

    let mut bounds_iter = bounds.into_iter();

    if let Some(bound) = bounds_iter.next() {
        let story_iter = database.stories.iter();

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

        first_push(include, &database, &mut stories, iter);
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

fn first_push<'d, I>(include: bool, database: &Database, stories: &mut Vec<Id>, ids: I)
where
    I: Iterator<Item = (&'d Id, &'d Story<StoryMetaRef>)>,
{
    if include {
        for id in ids.map(|(id, _)| id) {
            if !stories.contains(id) {
                stories.push(id.clone());
            }
        }
    } else {
        let ids = ids.map(|(id, _)| id).collect::<Vec<_>>();

        for id in database.stories.iter().map(|(id, _)| id) {
            if !ids.contains(&id) {
                stories.push(id.clone());
            }
        }
    }
}

fn any_by_text(full: &BTreeMap<Id, Entity>, refs: &[Id], text: &str) -> bool {
    refs.iter().map(|id| full.get(id)).any(|a| match a {
        Some(entity) => entity.text == text,
        None => false,
    })
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
