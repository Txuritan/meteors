use {
    crate::prelude::*,
    html_parser::{Dom, DomVariant, Node},
    std::{collections::HashMap, convert::TryFrom},
};

type Attributes<'input> = HashMap<&'input str, Option<&'input str>>;

pub struct Document<'input> {
    root: Dom<'input>,
}

impl<'input> Document<'input> {
    pub fn select(&self, selector: impl Into<Selector>) -> Vec<Node<'input>> {
        if self.root.tree_type == DomVariant::Document {
            let sel: Selector = selector.into();

            sel.find(&self.root.children)
        } else {
            vec![]
        }
    }
}

impl<'input> TryFrom<&'input str> for Document<'input> {
    type Error = anyhow::Error;

    fn try_from(input: &'input str) -> Result<Self, Self::Error> {
        let dom = Dom::parse(input)?;

        Ok(Self { root: dom })
    }
}

#[derive(Debug, PartialEq)]
pub struct Selector {
    matchers: Vec<Matcher>,
}

impl Selector {
    fn find_nodes<'input>(
        &self,
        matcher: &Matcher,
        elements: &[Node<'input>],
        direct_match: bool,
    ) -> Vec<Node<'input>> {
        let mut acc = vec![];

        for node in elements.iter() {
            if let Node::Element(el) = node {
                if !direct_match {
                    acc.append(&mut self.find_nodes(matcher, &el.children, false));
                }

                if matcher.matches(el.name, &el.attributes) {
                    acc.push(Node::Element(el.clone()));
                }
            }
        }

        acc
    }

    fn find<'input>(&self, elements: &[Node<'input>]) -> Vec<Node<'input>> {
        let mut elements: Vec<_> = elements.to_vec();
        let mut direct_match = false;

        for matcher in &self.matchers {
            if matcher.direct_match {
                direct_match = true;

                elements = elements
                    .iter()
                    .flat_map(|node| {
                        if let Node::Element(el) = node {
                            el.children.clone()
                        } else {
                            vec![]
                        }
                    })
                    .collect::<Vec<Node<'input>>>();

                continue;
            }

            elements = self.find_nodes(matcher, &elements, direct_match);
            direct_match = false;
        }

        elements.to_vec()
    }
}

impl From<&str> for Selector {
    fn from(input: &str) -> Self {
        let matchers: Vec<_> = input.split_whitespace().map(Matcher::from).collect();

        Selector { matchers }
    }
}

impl From<String> for Selector {
    fn from(input: String) -> Self {
        let matchers: Vec<_> = input.split_whitespace().map(Matcher::from).collect();

        Selector { matchers }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Matcher {
    tag: Vec<String>,
    class: Vec<String>,
    id: Vec<String>,
    attribute: HashMap<String, AttributeSpec>,
    direct_match: bool,
}

impl From<String> for Matcher {
    fn from(input: String) -> Self {
        Self::from(input.as_str())
    }
}

impl From<&str> for Matcher {
    fn from(input: &str) -> Self {
        let mut segments = vec![];
        let mut buf = "".to_string();

        for c in input.chars() {
            match c {
                '>' => {
                    return Self {
                        tag: vec![],
                        class: vec![],
                        id: vec![],
                        attribute: HashMap::new(),
                        direct_match: true,
                    };
                }
                '#' | '.' | '[' => {
                    segments.push(buf);
                    buf = "".to_string();
                }
                ']' => {
                    segments.push(buf);
                    buf = "".to_string();
                    continue;
                }
                _ => {}
            };

            buf.push(c);
        }
        segments.push(buf);

        let mut res = Self {
            tag: vec![],
            class: vec![],
            id: vec![],
            attribute: HashMap::new(),
            direct_match: false,
        };

        for segment in segments {
            match segment.chars().next() {
                Some('#') => res.id.push(segment[1..].to_string()),
                Some('.') => res.class.push(segment[1..].to_string()),
                Some('[') => res.add_data_attribute(segment[1..].to_string()),
                None => {}
                _ => res.tag.push(segment),
            }
        }

        res
    }
}

impl Matcher {
    fn add_data_attribute(&mut self, spec: String) {
        use AttributeSpec::*;

        let parts = spec.split('=').collect::<Vec<_>>();

        if parts.len() == 1 {
            let k = parts[0];
            self.attribute.insert(k.to_string(), Present);
            return;
        }

        let v = parts[1].trim_matches('"').to_string();
        let k = parts[0];
        let k = k[..k.len() - 1].to_string();

        match parts[0].chars().last() {
            Some('^') => {
                self.attribute.insert(k, Starts(v));
            }
            Some('$') => {
                self.attribute.insert(k, Ends(v));
            }
            Some('*') => {
                self.attribute.insert(k, Contains(v));
            }
            Some(_) => {
                let k = parts[0].to_string();
                self.attribute.insert(k, Exact(v));
            }
            None => {
                panic!("Could not parse attribute spec \"{}\"", spec);
            }
        }
    }

    fn matches<'input>(&self, name: &str, attrs: &Attributes<'input>) -> bool {
        let mut id_match = self.id.is_empty();
        if let Some(el_id) = attrs.get("id").copied().flatten() {
            let el_ids: Vec<_> = el_id.split_whitespace().collect();
            id_match = self.id.iter().all(|id| el_ids.iter().any(|eid| eid == id))
        }

        let mut class_match = self.class.is_empty();
        if let Some(el_class) = attrs.get("class").copied().flatten() {
            let el_classes: Vec<_> = el_class.split_whitespace().collect();

            class_match = self
                .class
                .iter()
                .all(|class| el_classes.iter().any(|eclass| eclass == class))
        }

        let mut attr_match = true;
        for (k, v) in &self.attribute {
            if let Some(value) = attrs.get(k.as_str()).copied().flatten() {
                if !v.matches(value) {
                    attr_match = false;
                    break;
                }
            }
        }

        let name = name.to_string();
        let tag_match = self.tag.is_empty() || self.tag.iter().any(|tag| &name == tag);

        tag_match && id_match && class_match && attr_match
    }
}

#[derive(Debug, PartialEq, Clone)]
enum AttributeSpec {
    Present,
    Exact(String),
    Starts(String),
    Ends(String),
    Contains(String),
}

impl AttributeSpec {
    fn matches(&self, other: &str) -> bool {
        use AttributeSpec::*;

        match self {
            Present => true,
            Exact(v) => other == v,
            Starts(v) => other.starts_with(v),
            Ends(v) => other.ends_with(v),
            Contains(v) => other.contains(v),
        }
    }
}
