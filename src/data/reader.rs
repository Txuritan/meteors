use {
    crate::{
        data::Database,
        models::proto::{Entity, Range, Rating, Story, StoryInfo, StoryMeta},
        prelude::*,
    },
    roxmltree::{Document, Node},
    std::{collections::BTreeMap, io::Read},
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

    let (authors, story_info) = read_info(&mut database.index.authors, &preface_meta)?;

    let preface_tags = preface_meta.get_child_by_class("tags")?;
    let story_meta = read_meta(authors, database, &preface_tags)?;

    let preface_message = preface.get_child_by_class("message")?;
    let story_id = read_id(&preface_message)?;

    let chapters = body.get_child_by_id("chapters")?;

    let mut chapter_sections = read_chapters(&chapters);

    if chapter_sections.is_empty() {
        let range = chapters.range();

        chapter_sections.push(Range::from_std(
            (range.start + "<div id=\"chapters\" class=\"userstuff\">".len())
                ..(range.end - "</div>".len()),
        ));
    }

    database.index.stories.insert(
        story_id,
        Story {
            file_name: name.to_string(),
            length: length as u32,
            chapters: chapter_sections,
            info: story_info,
            meta: story_meta,
        },
    );

    Ok(())
}

fn read_meta(
    authors: Vec<String>,
    database: &mut Database,
    node: &Node<'_, '_>,
) -> Result<StoryMeta> {
    let detail_names = children_elements(node).filter(|n| n.tag_name().name() == "dt");

    let detail_definitions = children_elements(node).filter(|n| n.tag_name().name() == "dd");

    let mut rating = Rating::Unknown;

    let mut categories = Vec::new();

    let mut origins = Vec::new();

    let mut warnings = Vec::new();
    let mut pairings = Vec::new();
    let mut characters = Vec::new();
    let mut generals = Vec::new();

    for (detail_names, detail_definition) in detail_names.zip(detail_definitions) {
        let part = match detail_names.get_text()?.trim() {
            "Rating:" => {
                match detail_definition.get_child("a")?.get_text()?.trim() {
                    "Explicit" => rating = Rating::Explicit,
                    "Mature" => rating = Rating::Mature,
                    "Teen And Up Audiences" => rating = Rating::Teen,
                    "Not Rated" => rating = Rating::NotRated,
                    _ => (),
                }

                None
            }
            "Archive Warning:" => Some((
                &mut database.index.warnings,
                &mut warnings,
                &detail_definition,
            )),
            "Category:" => Some((
                &mut database.index.categories,
                &mut categories,
                &detail_definition,
            )),
            "Fandom:" => Some((
                &mut database.index.origins,
                &mut origins,
                &detail_definition,
            )),
            "Relationship:" => Some((
                &mut database.index.pairings,
                &mut pairings,
                &detail_definition,
            )),
            "Characters:" => Some((
                &mut database.index.characters,
                &mut characters,
                &detail_definition,
            )),
            "Additional Tags:" => Some((
                &mut database.index.generals,
                &mut generals,
                &detail_definition,
            )),
            _ => None,
        };

        if let Some((map, list, node)) = part {
            add_children_to_list(map, list, node);
        }
    }

    Ok(StoryMeta {
        rating: Rating::to(rating),
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
    database_map: &mut BTreeMap<String, Entity>,
    node: &Node<'_, '_>,
) -> Result<(Vec<String>, StoryInfo)> {
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
    database_map: &mut BTreeMap<String, Entity>,
    list: &mut Vec<String>,
    node: &Node<'_, '_>,
) {
    for child in children_elements(node) {
        if let Some(text) = child.text() {
            add_to_if_exists_or_create(database_map, list, text);
        }
    }
}

fn add_to_if_exists_or_create(
    database_map: &mut BTreeMap<String, Entity>,
    list: &mut Vec<String>,
    text: &str,
) {
    let entry = database_map.iter().find(|(_, v)| v.text == text);

    if let Some((id, _)) = entry {
        list.push(id.clone());
    } else {
        let id = new_id();

        database_map.insert(
            id.clone(),
            Entity {
                text: text.to_string(),
            },
        );

        list.push(id);
    }
}

fn read_id(node: &Node<'_, '_>) -> Result<String> {
    let anchor = children_elements(node)
        .filter(|n| n.tag_name().name() == "a")
        .last();

    if let Some(anchor) = anchor {
        let text = anchor.get_attribute("href")?;
        let id = text.split('/').filter(|s| !s.is_empty()).last();

        id.map(String::from)
            .ok_or_else(|| anyhow!("could not find story id"))
    } else {
        Err(anyhow!("could not find original link"))
    }
}

fn read_chapters(node: &Node<'_, '_>) -> Vec<Range> {
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
        .map(Range::from_std)
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
            anyhow!(
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
                anyhow!(
                    "`{}` is missing the `{}` element",
                    self.tag_name().name(),
                    name,
                )
            })
    }

    fn get_child_by_class(&self, id: &str) -> Result<Node<'doc, 'input>> {
        children_elements(self)
            .find(|node| node.attribute("class") == Some(id))
            .ok_or_else(|| anyhow!("missing child with id `{}`", id))
    }

    fn get_child_by_id(&self, id: &str) -> Result<Node<'doc, 'input>> {
        children_elements(self)
            .find(|node| node.attribute("id") == Some(id))
            .ok_or_else(|| anyhow!("missing child with id `{}`", id))
    }

    fn get_text(&self) -> Result<&'doc str> {
        self.text()
            .ok_or_else(|| anyhow!("`{}` is missing body text", self.tag_name().name(),))
    }
}
