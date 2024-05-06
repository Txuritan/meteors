use {
    crate::{ParsedChapter, ParsedChapters, ParsedInfo, ParsedMeta},
    common::{models::Rating, prelude::*},
    query::{Document, Node, Span},
    std::ops::Range,
};

pub fn parse_info(doc: &Document<'_>) -> ParsedInfo {
    #[query::selector]
    static META_TITLE: &str = "html > body > #preface > .meta > h1";

    /// Selects the `byline` to get the story authors
    #[query::selector]
    static META_BYLINE: &str = "html > body > #preface > .meta > .byline > a[rel=author]";

    /// Selects the stories summary and notes
    #[query::selector]
    static META_SUMMARY: &str = "html > body > #preface > .meta > blockquote.userstuff";

    let title = doc
        .select(&META_TITLE)
        .and_then(Node::into_text)
        .unwrap_or("")
        .to_string();

    let summary = doc
        .select(&META_SUMMARY)
        .and_then(|node| {
            if node.children.is_empty() {
                None
            } else {
                let first = node.children.first().unwrap_or_else(|| unreachable!("BUG: There are no nodes in the vector even though it is no empty."));
                let last = node.children.last().unwrap_or_else(|| unreachable!("BUG: There is not 'last' node in the vector, this should at least return the first node."));

                Span::new(doc.input(), first.span.start(), last.span.end())
                    .map(|span| span.as_str())
            }
        })
        .unwrap_or("").to_string();

    let authors = {
        let authors: Option<Vec<String>> = doc
            .select_all(&META_BYLINE)
            .into_iter()
            .map(|n| n.get_text().map(String::from))
            .collect();

        match authors {
            Some(mut authors) => {
                if authors.is_empty() {
                    authors.push("Anonymous".to_string());
                }

                authors
            }
            None => {
                vec!["Anonymous".to_string()]
            }
        }
    };

    ParsedInfo {
        title,
        authors,
        summary,
    }
}

pub fn parse_meta(doc: &Document<'_>) -> ParsedMeta {
    #[query::selector]
    static META_TAGS_DT: &str = "html > body > #preface > .meta > .tags > dt";

    #[query::selector]
    static META_TAGS_DF: &str = "html > body > #preface > .meta > .tags > dd";

    let detail_names = doc.select_all(&META_TAGS_DT);
    let detail_definitions = doc.select_all(&META_TAGS_DF);

    let mut rating = Rating::Unknown;

    let mut categories = Vec::new();

    let mut origins = Vec::new();

    let mut warnings = Vec::new();
    let mut pairings = Vec::new();
    let mut characters = Vec::new();
    let mut generals = Vec::new();

    let nodes = detail_names.into_iter().zip(detail_definitions);

    for (detail_names, detail_definition) in nodes {
        let text = match detail_names.get_text().map(|text| text.trim()) {
            Some(text) => text.trim(),
            None => continue,
        };

        let list = match text {
            "Rating:" => {
                let text = detail_definition
                    .children
                    .first()
                    .and_then(|node| node.get_text().map(|text| text.trim()));

                match text {
                    Some("Explicit") => rating = Rating::Explicit,
                    Some("Mature") => rating = Rating::Mature,
                    Some("Teen And Up Audiences") => rating = Rating::Teen,
                    Some("General Audiences") => rating = Rating::General,
                    Some("Not Rated") => rating = Rating::NotRated,
                    _ => (),
                }

                None
            }
            "Archive Warning:" => Some(&mut warnings),
            "Category:" => Some(&mut categories),
            "Fandom:" => Some(&mut origins),
            "Relationship:" => Some(&mut pairings),
            "Character:" => Some(&mut characters),
            "Additional Tags:" => Some(&mut generals),
            _ => None,
        };

        if let Some(list) = list {
            for child in &detail_definition.children {
                if let Some(text) = child.get_text() {
                    list.push(text.to_string());
                }
            }
        }
    }

    ParsedMeta {
        rating,
        categories,
        origins,
        warnings,
        pairings,
        characters,
        generals,
    }
}

pub fn parse_chapters(doc: &Document<'_>) -> Result<ParsedChapters> {
    /// Selects the `toc-heading` that is present on single chapter stories
    #[query::selector]
    static CHAPTERS: &str = "html > body > #chapters";

    let chapter_node = doc
        .select(&CHAPTERS)
        .ok_or_else(|| anyhow!("BUG: There this no chapters block"))?;

    let chapters = match chapter_node.get_child_by_tag("h2") {
        Some(title_node) => parse_chapter_single(doc, title_node)
            .context("Story detected as having a single chapter")?,
        None => parse_chapters_multi(doc, chapter_node)
            .context("Story detected as having multiple chapters")?,
    };
    Ok(ParsedChapters { chapters })
}

#[inline]
fn parse_chapter_single<'input>(
    doc: &Document<'input>,
    title_node: &Node<'input>,
) -> Result<Vec<ParsedChapter>> {
    /// Selects the single chapter next to the `toc-heading`
    #[query::selector]
    static CHAPTER: &str = "html > body > #chapters > div.userstuff";

    match doc.select(&CHAPTER) {
        Some(chapter) => {
            // TODO: Do people have single chapter stories that have chapter summaries
            // let summary = toc_heading
            //     .get_child_by_tag("div")
            //     .and_then(|node| node.get_span_of_children(doc.input()))
            //     .map(|span| span.as_str());

            Ok(vec![ParsedChapter {
                title: title_node.get_text().map(String::from).ok_or_else(|| {
                    anyhow!(
                        "Story detected as having a single chapter, unable to find story title."
                    )
                })?,
                summary: None,
                start_notes: None,
                content: chapter
                    .get_span_of_children(doc.input())
                    .map(span_as_range)
                    .ok_or_else(|| {
                        anyhow!(
                            "Parser was unable to find chapter content for single chapter story"
                        )
                    })?,
                end_notes: None,
            }])
        }
        None => bail!("Story was detected as having a single chapter but none was found"),
    }
}

#[derive(Debug, Default)]
struct MultiState {
    title: Option<String>,
    summary: Option<String>,
    start_notes: Option<Range<usize>>,
    content: Option<Range<usize>>,
    end_notes: Option<Range<usize>>,
}

impl MultiState {
    #[inline]
    fn build(&mut self) -> Option<ParsedChapter> {
        self.title.take().and_then(|title| {
            self.content.take().map(|content| ParsedChapter {
                title,
                summary: self.summary.take(),
                start_notes: self.start_notes.take(),
                content,
                end_notes: self.end_notes.take(),
            })
        })
    }
}

#[inline]
fn parse_chapters_multi<'input>(
    doc: &Document<'input>,
    chapter_node: Node<'input>,
) -> Result<Vec<ParsedChapter>> {
    let (mut chapters, mut state) = chapter_node
        .children
        .iter()
        .filter(|n| n.is_element())
        .try_fold(
            (Vec::new(), MultiState::default()),
            |(mut chapters, mut state), node| -> Result<(_, _)> {
                let element = match node.get_element() {
                    Some(e) => e,
                    None => unreachable!(),
                };

                let classes = element
                    .get_attr("class")
                    .ok_or_else(|| anyhow!("Missing node classes, they should be there"))?;

                match classes {
                    // chapter details
                    "meta group" => {
                        if let Some(chapter) = state.build() {
                            chapters.push(chapter);
                        }

                        state.title = Some(
                            node.get_child_by_tag("h2")
                                .and_then(Node::get_text)
                                .map(String::from)
                                .ok_or_else(|| anyhow!("Unable to get chapter title"))?,
                        );

                        let p_nodes = node.get_children_by_tag("p").into_iter();
                        let blockquote_nodes = node.get_children_by_tag("blockquote").into_iter();

                        let nodes = p_nodes.zip(blockquote_nodes).fold(
                            (None, None),
                            |(mut summary, mut notes), (p, blockquote)| {
                                match p.get_text() {
                                    Some("Chapter Summary") => {
                                        summary = blockquote
                                            .get_span_of_children(doc.input())
                                            .map(|span| span.as_str());
                                    }
                                    Some("Chapter Notes") => {
                                        notes = blockquote.get_span_of_children(doc.input());
                                    }
                                    _ => {}
                                }

                                (summary, notes)
                            },
                        );

                        state.summary = nodes.0.map(String::from);
                        state.start_notes = nodes.1.map(span_as_range);
                    }
                    // chapter
                    "userstuff" => {
                        state.content = node.get_span_of_children(doc.input()).map(span_as_range);
                    }
                    // chapter end note
                    "meta" => {
                        state.end_notes = node
                            .get_child_by_tag("blockquote")
                            .and_then(|node| node.get_span_of_children(doc.input()))
                            .map(span_as_range);
                    }
                    _ => {}
                }

                Ok((chapters, state))
            },
        )?;

    if let Some(chapter) = state.build() {
        chapters.push(chapter);
    }

    Ok(chapters)
}

#[inline]
fn span_as_range(span: Span<'_>) -> Range<usize> {
    let start = span.start();
    let end = span.end();

    Range { start, end }
}
