const VOID_TAGS: [&str; 15] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta", "param",
    "source", "track", "wbr",
];

#[derive(Clone)]
pub struct Span<'input> {
    input: &'input str,
    start: usize,
    end: usize,
}

impl<'input> Span<'input> {
    pub fn new(input: &'input str, start: usize, end: usize) -> Self {
        Self { input, start, end }
    }

    fn shift(&mut self) {
        self.start = self.end;
    }

    pub fn as_str(&self) -> &'input str {
        &self.input[self.start..self.end]
    }

    fn ends_with(&self, pattern: &str) -> bool {
        &self.input[(pattern.len() - self.end)..self.end] == pattern
    }

    #[inline]
    fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::fmt::Debug for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

pub enum Error<'input> {
    AttrsDelim(&'input str),
    AttrsNotEqual(&'input str, Vec<&'input str>, Vec<&'input str>),
    AttrsNoKeyOrValue,
}

// Let's take `<img src="example.png" alt=image>` for example.
enum AttrPos {
    // This includes `src`, `alt`
    Key,
    // This includes `=`
    Equal,
    // This includes `example.png`, `image`
    Value(Option<char>),
    // This includes ` `
    Space,
}

// Valid `attr_str` like: `src="example.png" alt=example disabled`
fn attrs_parse<'input>(
    attr_str: &'input str,
    mut span: Span<'input>,
) -> Result<Vec<(&'input str, &'input str)>, Error<'input>> {
    let mut chars_stack: Vec<char> = Vec::new();
    let mut key_stack: Vec<&'input str> = Vec::new();
    let mut value_stack: Vec<&'input str> = Vec::new();
    let mut attr_pos = AttrPos::Key;

    for ch in attr_str.chars() {
        span.end += ch.len_utf8();

        match attr_pos {
            AttrPos::Key => match ch {
                '=' => {
                    attr_pos = AttrPos::Equal;

                    let key = span.as_str();
                    // let key = String::from_iter(chars_stack);

                    span.shift();
                    // chars_stack = Vec::new();

                    key_stack.push(key);
                }
                ' ' => {
                    attr_pos = AttrPos::Space;

                    let key = span.as_str();
                    // let key = String::from_iter(chars_stack);

                    span.shift();
                    // chars_stack = Vec::new();

                    key_stack.push(key);
                    value_stack.push("");
                }
                _ => chars_stack.push(ch),
            },
            AttrPos::Equal => match ch {
                '\'' => attr_pos = AttrPos::Value(Some('\'')),
                '\"' => attr_pos = AttrPos::Value(Some('\"')),
                _ => {
                    attr_pos = AttrPos::Value(None);
                    chars_stack.push(ch)
                }
            },
            AttrPos::Value(delimiter) => match delimiter {
                None => {
                    if ch == ' ' {
                        attr_pos = AttrPos::Space;

                        let value = span.as_str();
                        // let value = String::from_iter(chars_stack);

                        span.shift();
                        // chars_stack = Vec::new();

                        value_stack.push(value)
                    } else {
                        chars_stack.push(ch);
                    }
                }
                Some(quote) => {
                    if ch == quote {
                        if chars_stack.is_empty() {
                            value_stack.push("");
                            attr_pos = AttrPos::Space;

                            continue;
                        }
                        let last_char = chars_stack
                            .last()
                            .expect("cannot accesss the last char in `chars_stack`");
                        if last_char == &'\\' {
                            chars_stack.push(ch);

                            continue;
                        } else {
                            attr_pos = AttrPos::Space;

                            let value = span.as_str();
                            // let value = String::from_iter(chars_stack);

                            span.shift();
                            // chars_stack = Vec::new();

                            value_stack.push(value)
                        }
                    } else {
                        chars_stack.push(ch)
                    }
                }
            },
            AttrPos::Space => {
                if ch != ' ' {
                    attr_pos = AttrPos::Key;
                    chars_stack.push(ch);
                }
            }
        }
    }

    if !chars_stack.is_empty() {
        let str = span.as_str();
        // let str = String::from_iter(chars_stack);
        match attr_pos {
            AttrPos::Key => {
                key_stack.push(str);
                value_stack.push("");
            }
            AttrPos::Value(delimiter) => {
                if delimiter.is_none() {
                    value_stack.push(str);
                } else {
                    return Err(Error::AttrsDelim(attr_str));
                }
            }
            _ => {}
        }
    }

    if key_stack.len() != value_stack.len() {
        return Err(Error::AttrsNotEqual(attr_str, key_stack, value_stack));
    }

    let mut attrs = Vec::new();
    let len = key_stack.len();
    for _ in 0..len {
        attrs.push((
            key_stack.pop().ok_or(Error::AttrsNoKeyOrValue)?,
            value_stack.pop().ok_or(Error::AttrsNoKeyOrValue)?,
        ));
    }

    Ok(attrs)
}

#[derive(Debug, Clone)]
pub enum Token<'input> {
    // Like `<div>`, including `<img>`, `<input>`, etc.
    Start(&'input str, Vec<(&'input str, &'input str)>, Span<'input>),
    // Like `</div>`
    End(&'input str, Span<'input>),
    // Like `<div />`
    Closing(&'input str, Vec<(&'input str, &'input str)>, Span<'input>),
    // Like `<!doctype html>`
    Doctype(Span<'input>),
    // Like `<!-- comment -->`
    Comment(&'input str, Span<'input>),
    // Any text
    Text(&'input str, Span<'input>),
}

impl<'input> Token<'input> {
    fn from(tag: &'input str, span: Span<'input>) -> Option<Self> {
        if tag.ends_with("/>") {
            let tag_name_start = tag[1..tag.len()]
                .chars()
                .position(|x| x != ' ')
                .expect("tag name cannot be all spaces after \"<\"")
                + 1;

            let tag_name_end_option = tag[tag_name_start..tag.len()]
                .chars()
                .position(|x| x == ' ');

            let tag_name_end = match tag_name_end_option {
                Some(end) => end + tag_name_start,
                None => tag.len() - 2,
            };

            let tag_name = &tag[tag_name_start..tag_name_end];
            let attr_span = {
                let mut temp = span.clone();
                temp.end -= 2;
                temp
            };
            let attr_str = attr_span.as_str().trim();

            Some(Self::Closing(
                tag_name,
                attrs_parse(attr_str, attr_span).ok()?,
                span,
            ))
        } else if tag.starts_with("</") {
            Some(Self::End(tag[2..tag.len() - 1].trim(), span))
        } else if tag.starts_with("<!--") {
            Some(Self::from_comment(tag, span))
        } else if tag.starts_with("<!") {
            Some(Self::Doctype(span))
        } else if tag.starts_with('<') {
            let tag_name_start = tag[1..tag.len()]
                .chars()
                .position(|x| x != ' ')
                .expect("tag name cannot be all spaces after \"<\"")
                + 1;
            let tag_name_end_option = tag[tag_name_start..tag.len()]
                .chars()
                .position(|x| x == ' ');
            let tag_name_end = match tag_name_end_option {
                Some(end) => end + tag_name_start,
                None => tag.len() - 1,
            };

            let tag_name = &tag[tag_name_start..tag_name_end];
            let attr_span = {
                let mut temp = span.clone();
                temp.end -= 1;
                temp
            };
            let attr_str = attr_span.as_str().trim();

            Some(Self::Start(
                tag_name,
                attrs_parse(attr_str, attr_span).ok()?,
                span,
            ))
        } else {
            None
        }
    }

    #[inline]
    fn from_comment(comment: &'input str, span: Span<'input>) -> Self {
        Self::Comment(&comment[4..comment.len() - 3], span)
    }

    fn node(&self) -> Node<'input> {
        self.clone().into_node()
    }

    fn into_node(self) -> Node<'input> {
        let (data, span) = match self {
            Self::Start(name, attrs, span) => (
                NodeData::Element(Element {
                    name,
                    attrs,
                    children: Vec::new(),
                }),
                span,
            ),
            Self::End(name, span) => (
                NodeData::Element(Element {
                    name,
                    attrs: Vec::new(),
                    children: Vec::new(),
                }),
                span,
            ),
            Self::Closing(name, attrs, span) => (
                NodeData::Element(Element {
                    name,
                    attrs,
                    children: Vec::new(),
                }),
                span,
            ),
            Self::Doctype(span) => (NodeData::Doctype, span),
            Self::Comment(comment, span) => (NodeData::Comment(comment), span),
            Self::Text(text, span) => (NodeData::Text(text), span),
        };

        Node { data, span }
    }
}

/// Basic node of dom
#[derive(Debug, Clone)]
pub struct Node<'input> {
    pub data: NodeData<'input>,
    pub span: Span<'input>,
}

#[derive(Debug, Clone)]
pub enum NodeData<'input> {
    Element(Element<'input>),
    Text(&'input str),
    Comment(&'input str),
    Doctype,
}

impl<'input> Node<'input> {
    /// Check if it is an element node.
    ///
    /// ```
    /// use html_editor::Node;
    ///
    /// assert_eq!(Node::new_element("div", vec![("id", "app")], vec![]).is_element(), true);
    /// assert_eq!(Node::Text("Lorem Ipsum".to_string()).is_element(), false);
    /// ```
    pub fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    /// Convert the node into an element.
    ///
    /// Warning: The program will panic if it fails to convert.
    /// So take care to use this method unless you are sure.
    ///
    /// Example:
    /// ```
    /// use html_editor::{Node, Element};
    ///
    /// let a: Node = Node::new_element("div", vec![("id", "app")], vec![]);
    /// let a: Element = a.into_element();
    ///
    /// let b: Node = Node::Text("hello".to_string());
    /// // The next line will panic at 'Text("hello") is not an element'
    /// // let b: Element = a.into_element();
    /// ```
    pub fn into_element(self) -> Element<'input> {
        match self.data {
            NodeData::Element(element) => element,
            _ => panic!("{:?} is not an element", self),
        }
    }

    /// Create a new element node.
    ///
    /// ```
    /// use html_editor::Node;
    ///
    /// let node: Node = Node::new_element(
    ///     "h1",
    ///     vec![("class", "title")],
    ///     vec![
    ///         Node::Text("Hello, world!".to_string()),
    ///     ]
    /// );
    /// ```
    pub fn new_element(
        name: &'input str,
        attrs: Vec<(&'input str, &'input str)>,
        children: Vec<Node<'input>>,
        span: Span<'input>,
    ) -> Self {
        Node {
            data: NodeData::Element(Element {
                name,
                attrs,
                children,
            }),
            span,
        }
    }
}

/// HTML Element
#[derive(Debug, Clone)]
pub struct Element<'input> {
    pub name: &'input str,
    pub attrs: Vec<(&'input str, &'input str)>,
    pub children: Vec<Node<'input>>,
}

impl<'input> Element<'input> {
    /// Create a new element.
    pub fn new(
        name: &'input str,
        attrs: Vec<(&'input str, &'input str)>,
        children: Vec<Node<'input>>,
    ) -> Self {
        Self {
            name,
            attrs,
            children,
        }
    }
}

fn html_to_stack<'input>(html: &'input str) -> Result<Vec<Token<'input>>, String> {
    let mut span = Span::new(html, 0, 0);

    let mut chars_stack = Vec::<char>::new();
    let mut token_stack = Vec::<Token<'input>>::new();

    let mut in_quotes: Option<char> = None;
    // More precisely: is in angle brackets
    let mut in_brackets = false;
    let mut in_comment = false;
    let mut in_script = false;
    let mut in_style = false;

    for ch in html.chars() {
        dbg!((ch, &span, span.as_str()));

        if let Some(quote) = in_quotes {
            if ch == quote {
                let previous_char = *chars_stack
                    .last()
                    .expect("cannot get the last char in chars stack");
                if previous_char != '\\' {
                    in_quotes = None;
                }
            }

            span.end += ch.len_utf8();
            //chars_stack.push(ch);
        } else if in_comment {
            span.end += ch.len_utf8();
            // chars_stack.push(ch);

            if span.ends_with("-->") {
                // if String::from_iter(&chars_stack).ends_with("-->") {
                let comment = span.as_str();
                // let comment = String::from_iter(chars_stack);

                let comment_span = span.clone();

                span.shift();
                // chars_stack = Vec::new();

                token_stack.push(Token::from_comment(comment, comment_span));
                // token_stack.push(Token::from_comment(comment));

                in_comment = false;
                in_brackets = false;
            }
        } else if in_script {
            span.end += ch.len_utf8();
            // chars_stack.push(ch);

            // let len = chars_stack.len();

            if span.ends_with("</script>") {
                // if String::from_iter(&chars_stack).ends_with("</script>") {
                let script = &span.as_str()[..(span.len() - 9)];
                // let script = String::from_iter(chars_stack[..len - 9].to_vec());

                let script_src_span = {
                    let mut temp = span.clone();
                    temp.end -= 9;
                    temp
                };
                let script_span = span.clone();

                span.shift();
                // chars_stack = Vec::new();

                token_stack.push(Token::Text(script, script_src_span));
                token_stack.push(Token::End("script", script_span));

                in_script = false;
            }
        } else if in_style {
            span.end += ch.len_utf8();
            // chars_stack.push(ch);

            // let len = chars_stack.len();

            if span.ends_with("</style>") {
                // if String::from_iter(&chars_stack).ends_with("</style>") {
                let style = &span.as_str()[..(span.len() - 8)];
                // let style = String::from_iter(chars_stack[..len - 8].to_vec());

                let style_src_span = {
                    let mut temp = span.clone();
                    temp.end -= 8;
                    temp
                };
                let style_span = span.clone();

                span.shift();
                // chars_stack = Vec::new();

                token_stack.push(Token::Text(style, style_src_span));
                token_stack.push(Token::End("style", style_span));

                in_style = false;
            }
        } else {
            match ch {
                '<' => {
                    in_brackets = true;

                    // In case of pushing empty text tokens to the stack
                    if !chars_stack.is_empty() {
                        // Turn the chars in `chars_stack` into `String`
                        // and clean the chars stack.
                        let txt_text = span.as_str();
                        // let txt_text = String::from_iter(chars_stack);

                        let txt_span = span.clone();

                        span.shift();
                        // chars_stack = Vec::new();

                        // Push the text we just got to the token stack.
                        token_stack.push(Token::Text(txt_text, txt_span.clone()));
                    }

                    span.end += ch.len_utf8();
                    // chars_stack.push(ch);
                }
                '>' => {
                    in_brackets = false;

                    dbg!(in_brackets);

                    span.end += ch.len_utf8();
                    // chars_stack.push(ch);

                    // Turn the chars in `chars_stack` in to `String`
                    // and clean the chars stack.
                    let tag_text = span.as_str();
                    // let tag_text = String::from_iter(chars_stack);

                    let tag_span = span.clone();

                    span.shift();
                    // chars_stack = Vec::new();

                    // Push the tag with the text we just got to the token stack.
                    let tag = Token::from(tag_text, tag_span.clone())
                        .unwrap_or_else(|| panic!("Invalid tag: {}", tag_text));

                    token_stack.push(tag.clone());

                    // Handle special tags
                    if let Token::Start(tag_name, _, _) = tag {
                        match tag_name {
                            "script" => in_script = true,
                            "style" => in_style = true,
                            _ => {}
                        }
                    }
                }
                '-' => {
                    span.end += ch.len_utf8();
                    // chars_stack.push(ch);

                    if String::from_iter(&chars_stack) == "<!--" {
                        in_comment = true;
                    }
                }
                _ => {
                    if in_brackets {
                        match ch {
                            '\'' => in_quotes = Some('\''),
                            '\"' => in_quotes = Some('\"'),
                            _ => {}
                        }
                    }

                    span.end += ch.len_utf8();
                    // chars_stack.push(ch)
                }
            }
        }
    }

    if !chars_stack.is_empty() {
        let text = span.as_str();
        // let text = String::from_iter(chars_stack);

        token_stack.push(Token::Text(text, span.clone()));
    }

    Ok(token_stack)
}

fn stack_to_dom<'input>(token_stack: Vec<Token<'input>>) -> Result<Vec<Node<'input>>, String> {
    let mut nodes: Vec<Node<'input>> = Vec::new();
    let mut start_tags_stack: Vec<Token<'input>> = Vec::new();
    let mut start_tag_index = 0;

    for (i, token) in token_stack.iter().enumerate() {
        match token {
            Token::Start(tag, attrs, span) => {
                let is_void_tag = VOID_TAGS.contains(tag);

                if start_tags_stack.is_empty() {
                    if is_void_tag {
                        nodes.push(Node {
                            data: NodeData::Element(Element {
                                name: Clone::clone(tag),
                                attrs: attrs.clone(),
                                children: Vec::new(),
                            }),
                            span: span.clone(),
                        });
                    } else {
                        start_tag_index = i;
                        start_tags_stack.push(Token::Start(
                            Clone::clone(tag),
                            attrs.clone(),
                            span.clone(),
                        ));
                    }
                } else if is_void_tag {
                    // You do not need to push the void tag to the stack
                    // like above, because it must be inside the the
                    // element of the first start tag, and this element
                    // will then be pushed to the stack recursively.
                } else {
                    start_tags_stack.push(Token::Start(
                        Clone::clone(tag),
                        attrs.clone(),
                        span.clone(),
                    ));
                }
            }
            Token::End(tag, span) => {
                let start_tag = match start_tags_stack.pop() {
                    Some(token) => token.into_node().into_element(),
                    None => return Err(format!("No start tag matches </{}>", tag)),
                };

                if tag != &start_tag.name {
                    return Err(format!(
                        "<{}> does not match the </{}>",
                        start_tag.name, tag
                    ));
                }

                if start_tags_stack.is_empty() {
                    nodes.push(Node {
                        data: NodeData::Element(Element {
                            name: start_tag.name,
                            attrs: start_tag.attrs,
                            children: stack_to_dom(token_stack[start_tag_index + 1..i].to_vec())?,
                        }),
                        span: span.clone(),
                    })
                }
            }
            _ => {
                if start_tags_stack.is_empty() {
                    nodes.push(token.node());
                }
            }
        }
    }

    match start_tags_stack.pop() {
        Some(token) => {
            let start_tag_name = token.into_node().into_element().name;
            Err(format!("<{}> is not closed", start_tag_name))
        }
        None => Ok(nodes),
    }
}

fn try_stack_to_dom<'input>(token_stack: Vec<Token<'input>>) -> Vec<Node<'input>> {
    let mut nodes: Vec<Node<'input>> = Vec::new();
    let mut start_tags_stack: Vec<Token<'input>> = Vec::new();
    let mut start_tag_index = 0;

    for (i, token) in token_stack.iter().enumerate() {
        match token {
            Token::Start(tag, attrs, span) => {
                let is_void_tag = VOID_TAGS.contains(tag);

                if start_tags_stack.is_empty() {
                    if is_void_tag {
                        nodes.push(Node {
                            data: NodeData::Element(Element {
                                name: tag,
                                attrs: attrs.clone(),
                                children: Vec::new(),
                            }),
                            span: span.clone(),
                        });
                    } else {
                        start_tag_index = i;
                        start_tags_stack.push(Token::Start(tag, attrs.clone(), span.clone()));
                    }
                } else if is_void_tag {
                    // You do not need to push the void tag to the stack
                    // like above, because it must be inside the the
                    // element of the first start tag, and this element
                    // will then be pushed to the stack recursively.
                } else {
                    start_tags_stack.push(Token::Start(
                        Clone::clone(tag),
                        attrs.clone(),
                        span.clone(),
                    ));
                }
            }
            Token::End(tag, span) => {
                let start_tag = match start_tags_stack.pop() {
                    Some(token) => token.into_node().into_element(),
                    // It means the end tag is redundant, so we will omit
                    // it and just start the next loop.
                    None => continue,
                };

                if tag != &start_tag.name {
                    // The tags do not match, so let's put it back to
                    // pretend we never come here and then continue
                    // the next loop.
                    start_tags_stack.push(Token::Start(
                        start_tag.name,
                        start_tag.attrs,
                        span.clone(),
                    ));

                    continue;
                }

                if start_tags_stack.is_empty() {
                    nodes.push(Node {
                        data: NodeData::Element(Element {
                            name: start_tag.name,
                            attrs: start_tag.attrs,
                            children: try_stack_to_dom(
                                token_stack[start_tag_index + 1..i].to_vec(),
                            ),
                        }),
                        span: span.clone(),
                    })
                }
            }
            _ => {
                if start_tags_stack.is_empty() {
                    nodes.push(token.node());
                }
            }
        }
    }

    while let Some(token) = start_tags_stack.pop() {
        let node = match token {
            Token::Start(name, attrs, span) => Node {
                data: NodeData::Element(Element {
                    name,
                    attrs,
                    children: try_stack_to_dom(token_stack[start_tag_index + 1..].to_vec()),
                }),
                span: span.clone(),
            },
            _ => unreachable!(),
        };

        nodes = vec![node];
    }

    nodes
}

/// Parse the html string and return a `Vector` of `Node`.
///
/// Example:
/// ```
/// use html_editor::parse;
///
/// // Parse a segment.
/// let segment = parse(r#"<p class="content">Hello, world!</p>"#);
/// println!("{:#?}", segment);
///
/// // Or you can parse a whole html file.
/// let document = parse("<!doctype html><html><head></head><body></body></html>");
/// println!("{:#?}", document);
/// ```
///
/// Output:
/// ```log
/// [
///     Element {
///         name: "p",
///         attrs: {
///             "class": "content",
///         },
///         children: [
///             Text(
///                 "Hello, world!",
///             ),
///         ],
///     },
/// ]
/// [
///     Doctype,
///     Element {
///         name: "html",
///         attrs: {},
///         children: [
///             Element {
///                 name: "head",
///                 attrs: {},
///                 children: [],
///             },
///             Element {
///                 name: "body",
///                 attrs: {},
///                 children: [],
///             },
///         ],
///     },
/// ]
/// ```
pub fn parse(html: &str) -> Result<Vec<Node<'_>>, String> {
    let stack = html_to_stack(html)?;

    stack_to_dom(stack)
}

/// It's just like the function [`parse()`](parse), but with fault tolerance
/// future ---- whatever the input is, it will try to return a vector of nodes
/// without errors. It can parse some illegal html code  like `<div><a>Ipsum`
/// or `<div>Ipsum</a>`.
///
/// But we still suggest you to use [`parse()`](parse) unless neccessary for better
/// error handling.
pub fn try_parse(html: &str) -> Vec<Node<'_>> {
    let stack = html_to_stack(html).unwrap_or_default();

    try_stack_to_dom(stack)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn paired_tag() {
        // let a = parse("<p></p>");
        let b = parse("<div>Hello, world!</div>");

        // println!("{:#?}", a);
        println!("{:#?}", b);
    }
}
