#![deny(rust_2018_idioms)]

use std::{iter::Peekable, ops::Range, str::Chars};

fn main() {
    //     let text = r#"
    // enum (Option t) =
    // | (Some t)
    // | None

    // fn (option_is_some t) opt (Option t) bool =
    //     match opt =
    //     | (Some _) -> true
    //     | None -> false

    // enum (Result t e) =
    //     | (Ok t)
    //     | (Err e)

    // "#;

    //     println!("{:?}", Parser::new(text).parse_multiple::<Stmt<'_>>());

    let input = "(Option t)";

    assert_eq!(
        Ok(Type {
            name: Symbol(Span::new(input, 1..7)),
            generics: Some(vec![Type {
                name: Symbol(Span::new(input, 8..9)),
                generics: None,
                span: Span::new(input, 8..9),
            }]),
            span: Span::new(input, 0..10),
        }),
        Parser::new(input).parse::<Type<'_>>(),
    );
}

trait Parse<'i>: Sized {
    fn parse(parser: &mut Parser<'i>) -> Result<Self, Error>;
}

#[derive(Debug, PartialEq)]
enum Error {
    Char(char, char),
    Eof,
}

struct Parser<'i> {
    input: &'i str,
    chars: Peekable<Chars<'i>>,
    offset: usize,
}

impl<'i> Parser<'i> {
    fn new(input: &'i str) -> Self {
        let chars = input.chars().peekable();

        Self {
            input,
            chars,
            offset: 0,
        }
    }

    fn eat(&mut self) -> Result<(), Error> {
        while let Some(c) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }

            let c = self.chars.next().ok_or(Error::Eof)?;
            self.offset += c.len_utf8();
        }

        Ok(())
    }

    fn assert_c(exp: char, got: char) -> Result<(), Error> {
        if exp != got {
            return Err(Error::Char(exp, got));
        }

        Ok(())
    }

    fn next_c(&mut self) -> Result<char, Error> {
        let c = self.chars.next().ok_or(Error::Eof)?;

        self.offset += c.len_utf8();

        Ok(c)
    }

    fn next_i(&mut self) -> Result<Span<'i>, Error> {
        let start = self.offset;
        let mut end = self.offset;

        while let Some(c) = self.chars.next() {
            if c.is_whitespace() {
                self.offset += c.len_utf8();

                break;
            }

            end += c.len_utf8();
            self.offset += c.len_utf8();
        }

        Ok(self.new_span(start..end))
    }

    fn new_span(&self, range: Range<usize>) -> Span<'i> {
        Span {
            input: self.input,
            range,
        }
    }

    fn parse<P: Parse<'i>>(&mut self) -> Result<P, Error> {
        self.eat()?;

        let p = P::parse(self)?;

        self.eat()?;

        Ok(p)
    }

    fn parse_multiple<P: Parse<'i>>(&mut self) -> Result<Vec<P>, Error> {
        let mut parsed = Vec::new();

        self.eat()?;

        loop {
            if self.chars.peek().is_none() {
                break;
            }

            parsed.push(self.parse::<P>()?);

            if self.chars.peek().is_none() {
                break;
            }
        }

        Ok(parsed)
    }
}

#[derive(PartialEq)]
struct Span<'i> {
    input: &'i str,
    range: Range<usize>,
}

impl<'i> Span<'i> {
    fn new(input: &'i str, range: Range<usize>) -> Self {
        Self { input, range }
    }

    fn as_str(&self) -> &'i str {
        &self.input[self.range.clone()]
    }
}

impl<'i> std::fmt::Debug for Span<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("text", &&self.input[self.range.clone()])
            .field("range", &self.range)
            .finish()
    }
}

#[derive(Debug, PartialEq)]
struct Keyword<'i>(Span<'i>);

impl<'i> Keyword<'i> {
    fn new(span: Span<'i>) -> Self {
        Self(span)
    }
}

#[derive(Debug, PartialEq)]
struct Symbol<'i>(Span<'i>);

impl<'i> Symbol<'i> {
    fn new_from_span(span: Span<'i>) -> Self {
        Self(span)
    }
}

#[derive(Debug)]
enum Stmt<'i> {
    Fn(StmtFn<'i>),
    Enum(StmtEnum<'i>),
    Struct(StmtStruct<'i>),
}

impl<'i> Parse<'i> for Stmt<'i> {
    fn parse(parser: &mut Parser<'i>) -> Result<Self, Error> {
        let kind = parser.next_i()?;

        parser.eat()?;

        let stmt = match kind.as_str() {
            "fn" => Self::Fn(parser.parse()?),
            "enum" => Self::Enum(parser.parse()?),
            "struct" => Self::Struct(parser.parse()?),
            _ => todo!(),
        };

        Ok(stmt)
    }
}

#[derive(Debug)]
struct StmtFn<'i> {
    span: Span<'i>,
}

impl<'i> Parse<'i> for StmtFn<'i> {
    fn parse(parser: &mut Parser<'i>) -> Result<Self, Error> {
        let start = parser.offset;
        let end = parser.offset;

        Ok(Self {
            span: parser.new_span(start..end),
        })
    }
}

#[derive(Debug)]
struct StmtEnum<'i> {
    span: Span<'i>,
}

impl<'i> Parse<'i> for StmtEnum<'i> {
    fn parse(parser: &mut Parser<'i>) -> Result<Self, Error> {
        let start = parser.offset;
        let end = parser.offset;

        Ok(Self {
            span: parser.new_span(start..end),
        })
    }
}

#[derive(Debug)]
struct StmtStruct<'i> {
    span: Span<'i>,
}

impl<'i> Parse<'i> for StmtStruct<'i> {
    fn parse(parser: &mut Parser<'i>) -> Result<Self, Error> {
        let start = parser.offset;
        let end = parser.offset;

        Ok(Self {
            span: parser.new_span(start..end),
        })
    }
}

#[derive(Debug, PartialEq)]
struct Type<'i> {
    name: Symbol<'i>,
    generics: Option<Vec<Type<'i>>>,
    span: Span<'i>,
}

impl<'i> Parse<'i> for Type<'i> {
    fn parse(parser: &mut Parser<'i>) -> Result<Self, Error> {
        let start = parser.offset;

        let (name, generics) = match parser.chars.peek() {
            Some('(') => {
                Parser::assert_c('(', parser.next_c()?)?;

                let name = parser.next_i()?;

                let mut generics = Vec::new();

                loop {
                    if let Some(')') = parser.chars.peek() {
                        break;
                    }

                    generics.push(parser.parse::<Self>()?);
                }

                Parser::assert_c(')', parser.next_c()?)?;

                (name, Some(generics))
            }
            Some(_) => (parser.next_i()?, None),
            None => return Err(Error::Eof),
        };

        let end = parser.offset;

        Ok(Self {
            name: Symbol::new_from_span(name),
            generics,
            span: parser.new_span(start..end),
        })
    }
}

#[cfg(test)]
mod text_type {
    use super::{Parser, Span, Symbol, Type};

    #[test]
    fn simple() {
        let input = "Infallible";

        assert_eq!(
            Ok(Type {
                name: Symbol(Span::new(input, 0..10)),
                generics: None,
                span: Span::new(input, 0..10),
            }),
            Parser::new(input).parse::<Type<'_>>(),
        );
    }

    #[test]
    fn generic_1() {
        let input = "(Option t)";

        assert_eq!(
            Ok(Type {
                name: Symbol(Span::new(input, 1..7)),
                generics: Some(vec![Type {
                    name: Symbol(Span::new(input, 8..9)),
                    generics: None,
                    span: Span::new(input, 8..9),
                }]),
                span: Span::new(input, 0..10),
            }),
            Parser::new(input).parse::<Type<'_>>(),
        );
    }
}
