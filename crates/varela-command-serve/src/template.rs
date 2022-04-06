fn main() {
    let simple = Template::parse("Hello, {{ name }}!");
    println!("{:?}", simple);
}

use std::{ops::Range, rc::Rc, str::Chars};

#[derive(Debug, Clone)]
struct Template {
    elements: Rc<[Element]>,
}

impl Template {
    fn parse(input: &str) -> Self {
        let mut elements = Vec::new();

        let mut index = 0usize;
        let mut chars = input.chars();

        while let Some(element) = Self::next_element(input, &mut chars, &mut index) {
            elements.push(element);
        }

        Self {
            elements: Rc::from(elements),
        }
    }

    fn next_element(input: &str, chars: &mut Chars<'_>, index: &mut usize) -> Option<Element> {
        let start_index = *index;

        while let Some(c) = chars.next() {
            *index += c.len_utf8();

            if c == '{' {
                match chars.next() {
                    // handle insert expressions
                    Some('{') => {
                        *index += c.len_utf8();

                        // impl starts here
                        let start = (*index) - ('{'.len_utf8() * 2);
                        let mut end = *index;

                        while let Some(c) = chars.next() {
                            end += c.len_utf8();

                            if dbg!(c == '}') {
                                assert_eq!(Some('}'), chars.next());

                                end += '}'.len_utf8();

                                return Some(Element::ExprInsert(start..end));
                            }
                        }
                    }
                    // handle render expressions
                    Some('?') => {
                        *index += c.len_utf8();
                    }
                    // handle set expressions
                    Some('=') => {
                        *index += c.len_utf8();
                    }
                    // handle for statements
                    Some('#') => {
                        *index += c.len_utf8();
                    }
                    // handle if statements
                    Some('%') => {
                        *index += c.len_utf8();
                    }
                    Some(c) => *index += c.len_utf8(),
                    None => {}
                }
            }
        }

        None
    }
}

#[derive(Debug)]
enum Element {
    Raw(Range<usize>),

    // {{ var }}
    ExprInsert(Range<usize>),
    // {? var ?}
    ExprRender(Range<usize>),
    // {= var =}
    ExprSet(Range<usize>),

    // {# for var #} {# endfor #}
    StmtFor(Range<usize>, Vec<Element>),
    // {% if var1 ==/!= var1 %} {% endif %}
    StmtIf(bool, Range<usize>, Range<usize>, Vec<Element>),
}
