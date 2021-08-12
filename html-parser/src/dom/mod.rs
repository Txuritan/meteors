use crate::Result;
use pest::{iterators::Pairs, Parser, Span};

use crate::error::Error;
use crate::grammar::Grammar;
use crate::Rule;

pub mod element;
pub mod formatting;
pub mod node;

use element::{Element, ElementVariant};
use node::{Node, NodeData};

/// Document, DocumentFragment or Empty
#[derive(Debug, Clone, PartialEq)]
pub enum DomVariant {
    /// This means that the parsed html had the representation of an html document. The doctype is optional but a document should only have one root node with the name of html.
    /// Example:
    /// ```text
    /// <!doctype html>
    /// <html>
    ///     <head></head>
    ///     <body>
    ///         <h1>Hello world</h1>
    ///     </body>
    /// </html>
    /// ```
    Document,
    /// A document fragment means that the parsed html did not have the representation of a document. A fragment can have multiple root children of any name except html, body or head.
    /// Example:
    /// ```text
    /// <h1>Hello world</h1>
    /// ```
    DocumentFragment,
    /// An empty dom means that the input was empty
    Empty,
}

/// **The main struct** & the result of the parsed html
#[derive(Debug, Clone, PartialEq)]
pub struct Dom<'input> {
    /// The type of the tree that was parsed
    pub tree_type: DomVariant,

    /// All of the root children in the tree
    pub children: Vec<Node<'input>>,

    /// A collection of all errors during parsing
    pub errors: Vec<String>,
}

impl<'input> Default for Dom<'input> {
    fn default() -> Self {
        Self {
            tree_type: DomVariant::Empty,
            children: vec![],
            errors: vec![],
        }
    }
}

impl<'input> Dom<'input> {
    pub fn parse(input: &'input str) -> Result<Self> {
        let pairs = match Grammar::parse(Rule::html, input) {
            Ok(pairs) => pairs,
            Err(error) => return formatting::error_msg(error),
        };

        Self::build_dom(input, pairs)
    }

    fn build_dom(input: &'input str, pairs: Pairs<'input, Rule>) -> Result<Self> {
        let mut dom = Self::default();

        for pair in pairs {
            match pair.as_rule() {
                Rule::doctype_html | Rule::doctype_xml => {
                    dom.tree_type = DomVariant::DocumentFragment;
                }
                Rule::node_element => {
                    match Self::build_node_element(input, pair.into_inner(), &mut dom) {
                        Ok(el) => {
                            if let Some(node) = el {
                                dom.children.push(node);
                            }
                        }
                        Err(error) => {
                            dom.errors.push(format!("{}", error));
                        }
                    }
                }
                Rule::node_text => {
                    dom.children.push(Node::new(
                        NodeData::text(pair.as_str()),
                        vec![],
                        pair.as_span(),
                    ));
                }
                Rule::EOI => break,
                _ => unreachable!("[build dom] unknown rule: {:?}", pair.as_rule()),
            };
        }

        // TODO: This needs to be cleaned up
        // What logic should apply when parsing fragment vs document?
        // I had some of this logic inside the grammar before, but i thought it would be a bit clearer
        // to just have everyting here when we construct the dom
        match dom.children.len() {
            0 => {
                dom.tree_type = DomVariant::Empty;

                Ok(dom)
            }
            1 => match dom.children[0].data {
                NodeData::Element(ref el) => {
                    let name = el.name.to_lowercase();

                    if name == "html" {
                        dom.tree_type = DomVariant::Document;

                        Ok(dom)
                    } else if dom.tree_type == DomVariant::Document && name != "html" {
                        Err(Error::InvalidRoot)
                    } else {
                        dom.tree_type = DomVariant::DocumentFragment;

                        Ok(dom)
                    }
                }
                _ => {
                    dom.tree_type = DomVariant::DocumentFragment;

                    Ok(dom)
                }
            },
            _ => {
                dom.tree_type = DomVariant::DocumentFragment;

                for node in &dom.children {
                    if let NodeData::Element(ref el) = node.data {
                        let name = el.name.to_lowercase();

                        if name == "html" || name == "body" || name == "head" {
                            return Err(Error::IllegalInclude(name));
                        }
                    }
                }

                Ok(dom)
            }
        }
    }

    fn build_node_element(
        input: &'input str,
        pairs: Pairs<'input, Rule>,
        dom: &mut Dom,
    ) -> Result<Option<Node<'input>>> {
        let mut element = Element::default();

        let mut start = None;
        let mut end = None;

        let mut children = vec![];

        for pair in pairs {
            if start.is_none() {
                start = Some(pair.as_span());
            } else {
                end = Some(pair.as_span());
            }

            match pair.as_rule() {
                Rule::node_element | Rule::el_raw_text => {
                    match Self::build_node_element(input, pair.into_inner(), dom) {
                        Ok(el) => {
                            if let Some(child_element) = el {
                                children.push(child_element)
                            }
                        }
                        Err(error) => {
                            dom.errors.push(format!("{}", error));
                        }
                    }
                }
                Rule::node_text | Rule::el_raw_text_content => {
                    children.push(Node::new(
                        NodeData::text(pair.as_str()),
                        vec![],
                        pair.as_span(),
                    ));
                }
                // TODO: To enable some kind of validation we should probably align this with
                // https://html.spec.whatwg.org/multipage/syntax.html#elements-2
                // Also see element variants
                Rule::el_name | Rule::el_void_name | Rule::el_raw_text_name => {
                    element.name = pair.as_str();
                }
                Rule::attr => match Self::build_attribute(pair.into_inner()) {
                    Ok((attr_key, attr_value)) => {
                        element.attributes.insert(attr_key, attr_value);
                    }
                    Err(error) => {
                        dom.errors.push(format!("{}", error));
                    }
                },
                Rule::el_normal_end | Rule::el_raw_text_end => {
                    element.variant = ElementVariant::Normal;

                    break;
                }
                Rule::el_dangling => (),
                Rule::EOI => (),
                _ => {
                    return Err(Error::ElementCreation(Some(format!(
                        "{:?}",
                        pair.as_rule()
                    ))))
                }
            }
        }

        let span = match (start, end) {
            (Some(start), Some(end)) => {
                let span = Span::new(
                    input,
                    start.start() - 1,
                    end.end() + if !end.as_str().ends_with('>') { 1 } else { 0 },
                );

                unsafe { span.unwrap_unchecked() }
            }
            (Some(start), None) => start,
            _ => todo!(),
        };

        if !element.name.is_empty() {
            Ok(Some(Node::new(NodeData::element(element), children, span)))
        } else {
            Ok(None)
        }
    }

    fn build_attribute(pairs: Pairs<'input, Rule>) -> Result<(&'input str, Option<&'input str>)> {
        let mut attribute = ("", None);

        for pair in pairs {
            match pair.as_rule() {
                Rule::attr_key => {
                    attribute.0 = pair.as_str();
                }
                Rule::attr_value | Rule::attr_non_quoted => {
                    attribute.1 = Some(pair.as_str());
                }
                _ => return Err(Error::AttributeCreation(format!("{:?}", pair.as_rule()))),
            }
        }

        Ok(attribute)
    }
}
