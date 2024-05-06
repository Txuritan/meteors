use super::element::Element;
use pest::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Node<'input> {
    pub data: NodeData<'input>,

    /// All of the elements child nodes
    pub children: Vec<Node<'input>>,

    pub span: Span<'input>,
}

impl<'input> Node<'input> {
    pub(crate) fn new(
        data: NodeData<'input>,
        children: Vec<Node<'input>>,
        span: Span<'input>,
    ) -> Node<'input> {
        Self {
            data,
            children,
            span,
        }
    }

    pub fn get_attribute(&self, key: &str) -> Option<&'input str> {
        if let NodeData::Element(element) = &self.data {
            element.attributes.get(key).copied().flatten()
        } else {
            None
        }
    }

    pub fn get_child_by_tag(&self, tag: &str) -> Option<&Node<'input>> {
        self.children.iter().find(|node| {
            if let NodeData::Element(element) = &node.data {
                element.name == tag
            } else {
                false
            }
        })
    }

    pub fn get_children_by_tag(&self, tag: &str) -> Vec<&Node<'input>> {
        self.children
            .iter()
            .filter(|node| {
                if let NodeData::Element(element) = &node.data {
                    element.name == tag
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_span_of_children(&self, input: &'input str) -> Option<Span<'input>> {
        let first = self.children.first();
        let last = self.children.last();

        first
            .zip(last)
            .and_then(|(first, last)| Span::new(input, first.span.start(), last.span.end()))
    }

    pub fn get_text(&self) -> Option<&'input str> {
        self.children.first().and_then(|n| match n.data {
            NodeData::Text { contains } => Some(contains),
            _ => None,
        })
    }

    pub fn into_text(self) -> Option<&'input str> {
        self.get_text()
    }

    pub fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    pub fn get_element(&self) -> Option<&Element<'input>> {
        match &self.data {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeData<'input> {
    Comment { contains: &'input str },
    Element(Element<'input>),
    Text { contains: &'input str },
}

impl<'input> NodeData<'input> {
    pub(crate) fn element(element: Element<'input>) -> NodeData<'input> {
        NodeData::Element(element)
    }

    pub(crate) fn text(contains: &'input str) -> NodeData<'input> {
        NodeData::Text { contains }
    }
}

// impl<'a, 'input> IntoIterator for &'a Node<'input> {
//     type Item = &'a Node<'input>;
//     type IntoIter = NodeIntoIterator<'a, 'input>;

//     fn into_iter(self) -> Self::IntoIter {
//         NodeIntoIterator {
//             node: self,
//             index: vec![],
//         }
//     }
// }

pub struct NodeIntoIterator<'a, 'input> {
    node: &'a Node<'input>,
    // We add/remove to this vec each time we go up/down a node three
    index: Vec<(usize, &'a Node<'input>)>,
}

impl<'a, 'input> Iterator for NodeIntoIterator<'a, 'input> {
    type Item = &'a Node<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get first child
        let child = self.node.children.first();

        match child {
            // If element has child, return child
            Some(child) => {
                self.index.push((0, self.node));
                self.node = child;

                Some(child)
            }
            // If element doesn't have a child, but is a child of another node
            None if !self.index.is_empty() => {
                let mut has_finished = false;
                let mut next_node = None;

                while !has_finished {
                    // Try to get the next sibling of the parent node
                    if let Some((sibling_index, parent)) = self.index.pop() {
                        let next_sibling = sibling_index + 1;

                        let sibling = if let NodeData::Element(_) = parent.data {
                            parent.children.get(next_sibling)
                        } else {
                            None
                        };

                        if sibling.is_some() {
                            has_finished = true;

                            self.index.push((next_sibling, parent));

                            next_node = sibling;
                        } else {
                            continue;
                        }
                    // Break of there are no more parents
                    } else {
                        has_finished = true;
                    }
                }

                if let Some(next_node) = next_node {
                    self.node = next_node;
                }

                next_node
            }
            _ => None,
        }
    }
}
