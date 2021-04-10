use std::collections::HashMap;

/// Normal: `<div></div>` or Void: `<meta/>`and `<meta>`
#[derive(Debug, Clone, PartialEq)]
// TODO: Align with: https://html.spec.whatwg.org/multipage/syntax.html#elements-2
pub enum ElementVariant {
    /// A normal element can have children, ex: <div></div>.
    Normal,
    /// A void element can't have children, ex: <meta /> and <meta>
    Void,
}

pub type Attributes<'input> = HashMap<&'input str, Option<&'input str>>;

/// Most of the parsed html nodes are elements, except for text
#[derive(Debug, Clone, PartialEq)]
pub struct Element<'input> {
    /// The name / tag of the element
    pub name: &'input str,

    /// The element variant, if it is of type void or not
    pub variant: ElementVariant,

    /// All of the elements attributes, except id and class
    pub attributes: Attributes<'input>,
}

impl<'input> Default for Element<'input> {
    fn default() -> Self {
        Self {
            name: "",
            variant: ElementVariant::Void,
            attributes: HashMap::new(),
        }
    }
}
