#![deny(rust_2018_idioms)]

use std::{collections::HashMap, iter::Peekable, str::CharIndices};

#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter { expected: char, got: char },
    UnexpectedEof,
}

pub struct Dom<'input> {
    pub doctype: Option<Doctype>,
    pub children: Vec<Node<'input>>,
}

impl<'input> Dom<'input> {
    pub fn parse(input: &'input str) -> Result<Self, Error> {
        todo!()
    }
}

pub enum Doctype {
    Html5,
    Xhtml,
}

pub struct Node<'input> {
    pub children: Vec<Node<'input>>,
    pub span: Span<'input>,
}

pub enum NodeKind<'input> {
    Element(Element<'input>),
    Comment { text: &'input str },
    Text { text: &'input str },
}

pub type Attributes<'input> = HashMap<&'input str, Option<&'input str>>;

pub struct Element<'input> {
    pub tag: &'input str,
    pub kind: ElementKind,
    pub attributes: Attributes<'input>,
}

pub enum ElementKind {
    Normal,
    Void,
}

pub struct Span<'input> {
    pub input: &'input str,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
struct Parser<'input> {
    text: &'input str,
    reader: Peekable<CharIndices<'input>>,
    first: bool,
    current: Option<(usize, char)>,
}

impl<'input> Parser<'input> {
    fn new(text: &'input str) -> Self {
        Self {
            reader: text.char_indices().peekable(),
            text,
            first: true,
            current: None,
        }
    }

    fn current(&mut self) -> Option<&(usize, char)> {
        if self.first {
            self.current = self.reader.next();

            self.first = false;
        }

        self.current.as_ref()
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.current = self.reader.next();

        if self.first {
            self.first = false;
        }

        self.current().copied()
    }

    fn assert(&mut self, expected: char) -> Result<usize, Error> {
        if let Some((index, got)) = self.next() {
            if expected == got {
                Ok(index)
            } else {
                Err(Error::UnexpectedCharacter { expected, got })
            }
        } else {
            Err(Error::UnexpectedEof)
        }
    }

    #[allow(clippy::while_let_loop)]
    fn eat_whitespace(&mut self) -> Result<usize, Error> {
        let mut index = self
            .current()
            .map_or(Err(Error::UnexpectedEof), |(i, _)| Ok(*i))?;

        loop {
            match self.reader.peek() {
                Some((_, c)) => {
                    if !c.is_whitespace() {
                        break;
                    }
                }
                None => break,
            }

            let (i, _) = unsafe { self.next().unwrap_unchecked() };

            index = i;
        }

        Ok(index)
    }

    fn read_ident(&mut self) -> Result<&'input str, Error> {
        let index = self
            .current()
            .map_or(Err(Error::UnexpectedEof), |(i, _)| Ok(*i))?;

        let mut endex = index;

        loop {
            match self.reader.peek() {
                Some((_, '>')) => break,
                Some((_, c)) if c.is_whitespace() => break,
                Some(_) => {
                    if let Some((i, c)) = self.next() {
                        endex = i + c.len_utf8();
                    }
                }
                None => break,
            }
        }

        let name = &self.text[index..endex];

        Ok(name)
    }
}

#[cfg(test)]
mod test_read_ident {
    use super::Parser;

    #[test]
    fn simple() {
        let text = r#"attr"#;
        let mut parser = Parser::new(text);

        assert_eq!("attr", parser.read_ident().unwrap());
    }
}

impl<'input> Parser<'input> {
    fn read_attrs(&mut self) -> Result<Attributes<'input>, Error> {
        if self.reader.peek().map_or(false, |(_, c)| *c == '>') {
            return Ok(Attributes::new());
        }

        let mut attrs = Attributes::new();

        while let Some((key, value)) = self.read_attr()? {
            attrs.insert(key, value);
        }

        Ok(attrs)
    }

    fn read_attr(&mut self) -> Result<Option<(&'input str, Option<&'input str>)>, Error> {
        if self.reader.peek().map_or(false, |(_, c)| *c == '>') {
            return Ok(None);
        }

        self.eat_whitespace()?;

        // read key
        let key = self.read_ident()?;

        self.eat_whitespace()?;

        let value = if self.reader.peek().map_or(false, |(_, c)| *c == '=') {
            self.assert('=')?;

            self.eat_whitespace()?;

            // read value
            let value = self.read_ident()?;

            Some(value)
        } else {
            None
        };

        Ok(Some((key, value)))
    }
}

#[cfg(test)]
mod test_read_attrs {
    use super::{Attributes, Parser};

    #[test]
    fn single_solo() {
        let text = r#"attr"#;
        let mut parser = Parser::new(text);

        assert_eq!(
            {
                let mut map = Attributes::new();
                map.insert("attr", None);
                map
            },
            parser.read_attrs().unwrap()
        );
    }

    #[test]
    fn single_raw() {
        let text = r#"attr=true"#;
        let mut parser = Parser::new(text);

        assert_eq!(
            {
                let mut map = Attributes::new();
                map.insert("attr", Some("true"));
                map
            },
            parser.read_attrs().unwrap()
        );
    }

    #[test]
    fn single_quoted() {
        let text = r#"attr="true""#;
        let mut parser = Parser::new(text);

        assert_eq!(
            {
                let mut map = Attributes::new();
                map.insert("attr", Some("true"));
                map
            },
            parser.read_attrs().unwrap()
        );
    }
}

impl<'input> Parser<'input> {
    fn read_tag(&mut self) -> Result<(&'input str, bool, Attributes<'input>), Error> {
        self.assert('<')?;

        let name = self.read_ident()?;

        let attrs = self.read_attrs()?;

        self.assert('>')?;

        Ok((name, false, attrs))
    }
}

#[cfg(test)]
mod test_read_tag {
    use super::Parser;

    #[test]
    fn single() {
        let text = "<simple>";
        let mut parser = Parser::new(text);
        parser.read_tag().unwrap();
    }
}

