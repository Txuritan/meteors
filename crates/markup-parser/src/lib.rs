#![deny(rust_2018_idioms)]

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

struct Span<'input> {
    input: &'input str,
    start: usize,
    end: usize,
}

impl<'input> Span<'input> {
    pub fn as_str(&self) -> &'input str {
        &self.input[self.start..self.end]
    }
}

impl std::fmt::Debug for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("content", &self.as_str())
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

#[derive(Debug)]
enum Token<'input> {
    // HTML doctype/comment and XML cdata, `<!`
    Meta(Span<'input>),

    // XML prolog, `<?` and `?>`
    Prolog(Span<'input>),

    TagStart(Span<'input>),
    TagEnd(Span<'input>),

    Text(Span<'input>),

    Whitespace(Span<'input>),
}

fn spans<'input>(input: &'input str, index: &mut usize) -> Option<Token<'input>> {
    let start = *index;

    let mut chars = input[*index..].chars();

    let first = chars.next()?;
    *index += first.len_utf8();

    let is_chevron = first == '<';

    for c in chars {
        if is_chevron {
            *index += first.len_utf8();

            if c == '>' {
                let span = Span {
                    input,
                    start,
                    end: *index,
                };

                return Some(match span.as_str().chars().nth(1)? {
                    '/' => Token::TagEnd(span),
                    '!' => Token::Meta(span),
                    '?' => Token::Prolog(span),
                    _ => Token::TagStart(span),
                });
            }
        } else {
            if c == '<' {
                let span = Span {
                    input,
                    start,
                    end: *index,
                };

                if span.as_str().trim().is_empty() {
                    return Some(Token::Whitespace(span));
                } else {
                    return Some(Token::Text(span));
                }
            }

            *index += first.len_utf8();
        }
    }

    None
}

struct Dom<'input> {
    nodes: Vec<Node<'input>>,
}

impl<'input> Dom<'input> {
    fn push(&mut self, node: Node<'input>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }
}

type RcNode<'input> = Rc<Node<'input>>;
type WeakNode<'input> = Weak<Node<'input>>;

struct Node<'input> {
    data: NodeData<'input>,
    span: Span<'input>,
    parent: Option<RefCell<WeakNode<'input>>>,
    children: RefCell<Vec<Node<'input>>>,
}

impl<'input> Node<'input> {
    const fn new(
        parent: Option<RefCell<WeakNode<'input>>>,
        span: Span<'input>,
        data: NodeData<'input>,
    ) -> Self {
        Self {
            parent,
            data,
            span,
            children: RefCell::new(vec![]),
        }
    }
}

enum NodeData<'input> {
    Meta,
    Comment,

    Doctype,
    Prolog,

    Element { name: &'input str },
    Text,
}

fn parse<'input>(input: &'input str) -> Option<Dom<'input>> {
    let mut index = 0;

    let mut node: Option<RefCell<WeakNode<'input>>> = None;
    let mut dom = Dom { nodes: vec![] };

    while let Some(token) = spans(input, &mut index) {
        println!("{:?}", &token);

        match token {
            Token::Meta(span) => {
                let typ = match span.as_str().chars().nth(2) {
                    Some('D') | Some('d') => NodeData::Doctype,
                    Some('-') => NodeData::Comment,
                    _ => NodeData::Meta,
                };

                let _ = dom.push(Node::new(node.clone(), span, typ));
            }
            Token::Prolog(span) => {
                let _ = dom.push(Node::new(node.clone(), span, NodeData::Prolog));
            }

            Token::TagStart(span) => {}
            Token::TagEnd(span) => {}

            Token::Text(span) => {
                let _ = dom.push(Node::new(node.clone(), span, NodeData::Text));
            }
            Token::Whitespace(_) => {}
        }
    }

    println!();

    if dom.nodes.is_empty() {
        return None;
    }

    Some(dom)
}

fn main() {
    parse(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    parse(r#"<!DOCTYPE html>"#);
    parse(
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">"#,
    );
    parse(r#"<!-- Write your comments here -->"#);
    parse(r#"<div></div>"#);
    parse(r#"<div>Hello, world!</div>"#);
    parse(
        r#"
        <div>Hello, world!</div>
        <p>Botton Text</p>
    "#,
    );
}
