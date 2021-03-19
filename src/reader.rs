use {
    crate::{
        database::{Database, Id},
        models::{Entity, Rating, Story, StoryInfo, StoryMetaRef},
        prelude::*,
    },
    roxmltree::{Document, Node},
    std::{collections::BTreeMap, io::Read, ops::Range},
};

pub fn read_story<R>(database: &mut Database, name: &str, reader: &mut R) -> Result<()>
where
    R: Read,
{
    let mut buf = String::new();

    let _ = reader.read_to_string(&mut buf)?;

    let length = buf.len();

    let doc = Document::parse(&buf)?;

    let html = doc.root_element();
    let body = html.get_child("body")?;

    let preface = body.get_child_by_id("preface")?;
    let preface_meta = preface.get_child_by_class("meta")?;

    let (authors, story_info) = read_info(&mut database.authors, &preface_meta)?;

    let preface_tags = preface_meta.get_child_by_class("tags")?;
    let story_meta = read_meta(authors, database, &preface_tags)?;

    let preface_message = preface.get_child_by_class("message")?;
    let story_id = read_id(&preface_message)?;

    let chapters = body.get_child_by_id("chapters")?;

    let mut chapter_sections = read_chapters(&chapters);

    if chapter_sections.is_empty() {
        let range = chapters.range();

        chapter_sections.push(
            (range.start + "<div id=\"chapters\" class=\"userstuff\">".len())
                ..(range.end - "</div>".len()),
        );
    }

    database.stories.insert(
        story_id,
        Story {
            file_name: name.to_string(),
            length,
            chapters: chapter_sections,
            info: story_info,
            meta: story_meta,
        },
    );

    Ok(())
}

fn read_meta(
    authors: Vec<Id>,
    database: &mut Database,
    node: &Node<'_, '_>,
) -> Result<StoryMetaRef> {
    let dts = node
        .children()
        .filter(|n| n.is_element())
        .filter(|n| n.tag_name().name() == "dt");

    let dds = node
        .children()
        .filter(|n| n.is_element())
        .filter(|n| n.tag_name().name() == "dd");

    let mut rating = Rating::Unknown;

    let mut categories = Vec::new();

    let mut origins = Vec::new();

    let mut warnings = Vec::new();
    let mut pairings = Vec::new();
    let mut characters = Vec::new();
    let mut generals = Vec::new();

    for (dt, dd) in dts.zip(dds) {
        let part = match dt.get_text()?.trim() {
            "Rating:" => {
                match dd.get_child("a")?.get_text()?.trim() {
                    "Explicit" => rating = Rating::Explicit,
                    "Mature" => rating = Rating::Mature,
                    "Teen And Up Audiences" => rating = Rating::Teen,
                    "Not Rated" => rating = Rating::NotRated,
                    _ => (),
                }

                None
            }
            "Archive Warning:" => Some((&mut database.warnings, &mut warnings, &dd)),
            "Category:" => Some((&mut database.categories, &mut categories, &dd)),
            "Fandom:" => Some((&mut database.origins, &mut origins, &dd)),
            "Relationship:" => Some((&mut database.pairings, &mut pairings, &dd)),
            "Characters:" => Some((&mut database.characters, &mut characters, &dd)),
            "Additional Tags:" => Some((&mut database.generals, &mut generals, &dd)),
            _ => None,
        };

        if let Some((map, list, node)) = part {
            add_children_to_list(map, list, node);
        }
    }

    Ok(StoryMetaRef {
        rating,
        categories,
        authors,
        origins,
        warnings,
        pairings,
        characters,
        generals,
    })
}

fn read_info(
    database_map: &mut BTreeMap<Id, Entity>,
    node: &Node<'_, '_>,
) -> Result<(Vec<Id>, StoryInfo)> {
    let title_node = node.get_child("h1")?;
    let authors_node = node.get_child_by_class("byline")?;
    let summary = node
        .get_child("blockquote")
        .and_then(|node| Ok(node.get_child("p")?.get_text()?.to_string()))
        .unwrap_or_else(|_| String::new());

    let mut authors = Vec::new();

    add_children_to_list(database_map, &mut authors, &authors_node);

    if authors.is_empty() {
        add_to_if_exists_or_create(database_map, &mut authors, "Anonymous");
    }

    Ok((
        authors,
        StoryInfo {
            title: title_node.get_text()?.to_string(),
            summary,
        },
    ))
}

fn add_children_to_list(
    database_map: &mut BTreeMap<Id, Entity>,
    list: &mut Vec<Id>,
    node: &Node<'_, '_>,
) {
    for child in children_elements(node) {
        if let Some(text) = child.text() {
            add_to_if_exists_or_create(database_map, list, text);
        }
    }
}

fn add_to_if_exists_or_create(
    database_map: &mut BTreeMap<Id, Entity>,
    list: &mut Vec<Id>,
    text: &str,
) {
    let entry = database_map.iter().find(|(_, v)| v.text == text);

    match entry {
        Some((id, _)) => {
            list.push(id.clone());
        }
        None => {
            let id = Id::new_rand();

            database_map.insert(
                id.clone(),
                Entity {
                    text: text.to_string(),
                },
            );

            list.push(id);
        }
    }
}

fn read_id(node: &Node<'_, '_>) -> Result<Id> {
    let anchor = children_elements(node)
        .filter(|n| n.tag_name().name() == "a")
        .last();

    if let Some(anchor) = anchor {
        let text = anchor.get_attribute("href")?;
        let id = text.split('/').filter(|s| !s.is_empty()).last();

        id.map(Id::from_str)
            .ok_or_else(|| eyre!("could not find story id"))
    } else {
        Err(eyre!("could not find original link"))
    }
}

fn read_chapters(node: &Node<'_, '_>) -> Vec<Range<usize>> {
    let meta_groups =
        children_elements(node).filter(|n| n.attribute("class") == Some("meta group"));

    let userstuffs = children_elements(node).filter(|n| n.attribute("class") == Some("userstuff"));

    meta_groups
        .zip(userstuffs)
        .map(|(meta_group, userstuff)| {
            let start = meta_group.range().start;
            let end = userstuff.range().end;

            start..end
        })
        .collect::<Vec<_>>()
}

fn children_elements<'node, 'doc, 'input>(
    node: &'node Node<'doc, 'input>,
) -> impl Iterator<Item = Node<'doc, 'input>> {
    node.children().filter(Node::is_element)
}

trait NodeExt<'doc, 'input> {
    fn get_attribute(&self, name: &str) -> Result<&'doc str>;
    fn get_child(&self, name: &str) -> Result<Node<'doc, 'input>>;
    fn get_child_by_class(&self, id: &str) -> Result<Node<'doc, 'input>>;
    fn get_child_by_id(&self, id: &str) -> Result<Node<'doc, 'input>>;
    fn get_text(&self) -> Result<&'doc str>;
}

impl<'doc, 'input> NodeExt<'doc, 'input> for Node<'doc, 'input> {
    fn get_attribute(&self, name: &str) -> Result<&'doc str> {
        self.attribute(name).ok_or_else(|| {
            eyre!(
                "`{}` is missing the `{}` attribute",
                self.tag_name().name(),
                name,
            )
        })
    }

    fn get_child(&self, name: &str) -> Result<Node<'doc, 'input>> {
        children_elements(self)
            .find(|n| n.tag_name().name() == name)
            .ok_or_else(|| {
                eyre!(
                    "`{}` is missing the `{}` element",
                    self.tag_name().name(),
                    name,
                )
            })
    }

    fn get_child_by_class(&self, id: &str) -> Result<Node<'doc, 'input>> {
        children_elements(self)
            .find(|node| node.attribute("class") == Some(id))
            .ok_or_else(|| eyre!("missing child with id `{}`", id))
    }

    fn get_child_by_id(&self, id: &str) -> Result<Node<'doc, 'input>> {
        children_elements(self)
            .find(|node| node.attribute("id") == Some(id))
            .ok_or_else(|| eyre!("missing child with id `{}`", id))
    }

    fn get_text(&self) -> Result<&'doc str> {
        self.text()
            .ok_or_else(|| eyre!("`{}` is missing body text", self.tag_name().name(),))
    }
}
