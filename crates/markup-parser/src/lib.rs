#![deny(rust_2018_idioms)]

use honeycomb::{
    atoms::{one_of, opt, space, sym},
    language::{alpha, alphanumeric, string},
    transform::collect,
    Parser,
};

#[derive(Debug, Clone, PartialEq)]
struct Ident(String);

fn parse_ident() -> Parser<Ident> {
    ((alpha() | one_of(b"@")).is() >> (((alphanumeric() | one_of(b"@._-")) * (1..31)) - collect))
        % "an identifier"
        - Ident
}

#[cfg(test)]
mod test_ident {
    use super::{parse_ident, test_data, Ident};

    #[test]
    fn parse_html_attributes() {
        for attribute in test_data::HTML_ATTRIBUTES {
            let expected = Ident(attribute.to_string());
            let got = parse_ident().parse(attribute).unwrap();
            assert_eq!(expected, got, "with attribute: {}", attribute);
        }
    }

    #[test]
    fn parse_html_tags() {
        for tag in test_data::HTML_TAGS {
            let expected = Ident(tag.to_string());
            let got = parse_ident().parse(tag).unwrap();
            assert_eq!(expected, got, "with tag: {}", tag);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AttributeValue {
    Quoted(String),
    Raw(String),
}

fn parse_attribute_value() -> Parser<AttributeValue> {
    let parse_quoted = string() % "a quoted attribute value" - AttributeValue::Quoted;
    let parse_raw =
        ((alphanumeric() * (..)) - collect) % "a raw attribute value" - AttributeValue::Raw;

    parse_quoted | parse_raw
}

#[cfg(test)]
mod test_attribute_value {
    use super::{parse_attribute_value, test_data, AttributeValue};

    #[test]
    fn parse_html_attribute_values_raw() {
        for value in test_data::HTML_ATTRIBUTE_VALUES_RAW {
            let expected = AttributeValue::Raw(value.to_string());
            let got = parse_attribute_value().parse(value).unwrap();
            assert_eq!(expected, got, "with value: {}", value);
        }
    }

    #[test]
    fn parse_html_attribute_values_quoted() {
        for value in test_data::HTML_ATTRIBUTE_VALUES_QUOTED {
            let expected = AttributeValue::Quoted(
                value
                    .trim_start_matches('\"')
                    .trim_end_matches('\"')
                    .to_string(),
            );
            let got = parse_attribute_value().parse(value).unwrap();
            assert_eq!(expected, got, "with value: {}", value);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Attribute(Ident, Option<AttributeValue>);

impl Attribute {
    fn from((i, o): (Ident, Option<AttributeValue>)) -> Self {
        Self(i, o)
    }
}

fn parse_attribute() -> Parser<Attribute> {
    (parse_ident() & opt(sym('=') >> parse_attribute_value()))
        % "expected a attribute key or key value pair"
        - Attribute::from
}

#[cfg(test)]
mod test_attribute {
    use super::{parse_attribute, test_data, Attribute, AttributeValue, Ident};

    #[test]
    fn parse_html_attribute_empty() {
        for key in test_data::HTML_ATTRIBUTES {
            let expected = Attribute(Ident(key.to_string()), None);
            let got = parse_attribute().parse(key).unwrap();
            assert_eq!(expected, got, "with attribute: {}", key);
        }
    }

    #[test]
    fn parse_html_attribute_pair_raw() {
        for key in test_data::HTML_ATTRIBUTES {
            for value in test_data::HTML_ATTRIBUTE_VALUES_RAW {
                let attribute = format!("{}={}", key, value);

                let expected = Ok(Attribute(
                    Ident(key.to_string()),
                    Some(AttributeValue::Raw(value.to_string())),
                ));

                let got = parse_attribute().parse(&attribute);

                assert_eq!(expected, got, "with attribute: {}", attribute);
            }
        }
    }

    #[test]
    fn parse_html_attribute_pair_quoted() {
        for key in test_data::HTML_ATTRIBUTES {
            for value in test_data::HTML_ATTRIBUTE_VALUES_QUOTED {
                let attribute = format!("{}={}", key, value);

                let expected = Ok(Attribute(
                    Ident(key.to_string()),
                    Some(AttributeValue::Quoted(
                        value
                            .trim_start_matches('\"')
                            .trim_end_matches('\"')
                            .to_string(),
                    )),
                ));

                let got = parse_attribute().parse(&attribute);

                assert_eq!(expected, got, "with attribute: {}", attribute);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AttributeList(Vec<Attribute>);

fn parse_attribute_list() -> Parser<AttributeList> {
    ((space() >> parse_attribute() << space()) * (..)) % "expected node attribute list"
        - AttributeList
}

#[cfg(test)]
mod test_attribute_list {
    use super::{parse_attribute_list, test_data, Attribute, AttributeList, AttributeValue, Ident};

    #[test]
    fn parse_html_node_attribute_empty() {
        for attr1 in test_data::HTML_ATTRIBUTES {
            for attr2 in test_data::HTML_ATTRIBUTES {
                let attribute = format!("{} {}", attr1, attr2);

                let expected = Ok(AttributeList(vec![
                    Attribute(Ident(attr1.to_string()), None),
                    Attribute(Ident(attr2.to_string()), None),
                ]));

                let got = parse_attribute_list().parse(&attribute);

                assert_eq!(expected, got, "with attribute list: {}", attribute);
            }
        }
    }

    #[test]
    fn parse_html_node_attribute_raw() {
        for attr1 in test_data::HTML_ATTRIBUTES {
            for value1 in test_data::HTML_ATTRIBUTE_VALUES_RAW {
                for attr2 in test_data::HTML_ATTRIBUTES {
                    for value2 in test_data::HTML_ATTRIBUTE_VALUES_RAW {
                        let attribute = format!("{}={} {}={}", attr1, value1, attr2, value2);

                        let expected = Ok(AttributeList(vec![
                            Attribute(
                                Ident(attr1.to_string()),
                                Some(AttributeValue::Raw(value1.to_string())),
                            ),
                            Attribute(
                                Ident(attr2.to_string()),
                                Some(AttributeValue::Raw(value2.to_string())),
                            ),
                        ]));

                        let got = parse_attribute_list().parse(&attribute);

                        assert_eq!(expected, got, "with attribute list: {}", attribute);
                    }
                }
            }
        }
    }

    #[test]
    fn parse_html_node_attribute_quoted() {
        for attr1 in test_data::HTML_ATTRIBUTES {
            for value1 in test_data::HTML_ATTRIBUTE_VALUES_QUOTED {
                for attr2 in test_data::HTML_ATTRIBUTES {
                    for value2 in test_data::HTML_ATTRIBUTE_VALUES_QUOTED {
                        let attribute = format!("{}={} {}={}", attr1, value1, attr2, value2);

                        let expected = Ok(AttributeList(vec![
                            Attribute(
                                Ident(attr1.to_string()),
                                Some(AttributeValue::Quoted(
                                    value1
                                        .trim_start_matches('\"')
                                        .trim_end_matches('\"')
                                        .to_string(),
                                )),
                            ),
                            Attribute(
                                Ident(attr2.to_string()),
                                Some(AttributeValue::Quoted(
                                    value2
                                        .trim_start_matches('\"')
                                        .trim_end_matches('\"')
                                        .to_string(),
                                )),
                            ),
                        ]));

                        let got = parse_attribute_list().parse(&attribute);

                        assert_eq!(expected, got, "with attribute list: {}", attribute);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NodeOpen(Ident, AttributeList);

impl NodeOpen {
    fn from((i, a): (Ident, AttributeList)) -> Self {
        Self(i, a)
    }
}

fn parse_node_open() -> Parser<NodeOpen> {
    (sym('<') >> (parse_ident() & (space() >> parse_attribute_list())) << sym('>'))
        % "expected node opening declaration"
        - NodeOpen::from
}

#[cfg(test)]
mod test_node_open {
    use super::{
        parse_node_open, test_data, Attribute, AttributeList, AttributeValue, Ident, NodeOpen,
    };

    #[test]
    fn parse_html_node_attribute_empty() {
        for tag in test_data::HTML_TAGS {
            for key in test_data::HTML_ATTRIBUTES {
                let attribute = format!("<{} {}>", tag, key);

                let expected = Ok(NodeOpen(
                    Ident(tag.to_string()),
                    AttributeList(vec![Attribute(Ident(key.to_string()), None)]),
                ));

                let got = parse_node_open().parse(&attribute);

                assert_eq!(expected, got, "with node open: {}", attribute);
            }
        }
    }

    #[test]
    fn parse_html_node_attribute_raw() {
        for tag in test_data::HTML_TAGS {
            for key in test_data::HTML_ATTRIBUTES {
                for value in test_data::HTML_ATTRIBUTE_VALUES_RAW {
                    let attribute = format!("<{} {}={}>", tag, key, value);

                    let expected = Ok(NodeOpen(
                        Ident(tag.to_string()),
                        AttributeList(vec![Attribute(
                            Ident(key.to_string()),
                            Some(AttributeValue::Raw(value.to_string())),
                        )]),
                    ));

                    let got = parse_node_open().parse(&attribute);

                    assert_eq!(expected, got, "with node open: {}", attribute);
                }
            }
        }
    }

    #[test]
    fn parse_html_node_attribute_quoted() {
        for tag in test_data::HTML_TAGS {
            for key in test_data::HTML_ATTRIBUTES {
                for value in test_data::HTML_ATTRIBUTE_VALUES_QUOTED {
                    let attribute = format!("<{} {}={}>", tag, key, value);

                    let expected = Ok(NodeOpen(
                        Ident(tag.to_string()),
                        AttributeList(vec![Attribute(
                            Ident(key.to_string()),
                            Some(AttributeValue::Quoted(
                                value
                                    .trim_start_matches('\"')
                                    .trim_end_matches('\"')
                                    .to_string(),
                            )),
                        )]),
                    ));

                    let got = parse_node_open().parse(&attribute);

                    assert_eq!(expected, got, "with node open: {}", attribute);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NodeClose(Ident);

fn prase_node_close() -> Parser<NodeClose> {
    (sym('<') >> (sym('/') >> parse_ident()) << sym('>')) % "expected node closing declaration"
        - NodeClose
}

#[cfg(test)]
mod test_node_close {}

#[cfg(test)]
mod test_data {
    #[rustfmt::skip]
    pub(crate) static HTML_TAGS: &[&str] = &[
        "a", "abbr", "acronym", "address", "applet", "area", "article", "aside", "audio",
        "b", "base", "basefont", "bdi", "bdo", "big", "blink", "blockquote", "body", "br",
        "button", "canvas", "caption", "center", "cite", "code", "col", "colgroup", "comment",
        "datalist", "dd", "del", "details", "dfn", "dialog", "dir", "div", "dl", "dt", "em",
        "embed", "fieldset", "figcaption", "figure", "font", "footer", "form", "frame",
        "frameset", "h1", "head", "header", "hr", "html", "i", "iframe", "img", "input", "ins",
        "kbd", "keygen", "label", "legend", "li", "link", "main", "map", "mark", "menu",
        "menuitem", "meta", "meter", "nav", "noframes", "noscript", "object", "ol", "optgroup",
        "option", "output", "p", "param", "pre", "progress", "q", "rp", "rt", "ruby", "s", "samp",
        "script", "section", "select", "small", "source", "span", "strike", "strong", "style",
        "sub", "summary", "sup", "table", "tbody", "td", "tfoot", "th", "thead", "time", "title",
        "tr", "track", "tt", "u", "ul", "var", "video", "wbr",
    ];

    #[rustfmt::skip]
    pub(crate) static HTML_ATTRIBUTES: &[&str] = &[
        "accept", "accept-charset", "accesskey", "action", "align", "allow", "alt", "async",
        "autocapitalize", "autocomplete", "autofocus", "autoplay", "background", "bgcolor",
        "border", "buffered", "capture", "challenge", "charset", "checked", "cite", "class",
        "code", "codebase", "color", "cols", "colspan", "content", "contenteditable",
        "contextmenu", "controls", "coords", "crossorigin", "csp", "data", "datetime", "decoding",
        "default", "defer", "dir", "dirname", "disabled", "download", "draggable", "enctype",
        "enterkeyhint", "for", "form", "formaction", "formenctype", "formmethod", "formnovalidate",
        "formtarget", "headers", "height", "hidden", "high", "href", "hreflang", "http-equiv",
        "icon", "id", "importance", "integrity", "intrinsicsize", "inputmode", "ismap", "itemprop",
        "keytype", "kind", "label", "lang", "language", "loading", "list", "loop", "low",
        "manifest", "max", "maxlength", "minlength", "media", "method", "min", "multiple", "muted",
        "name", "novalidate", "open", "optimum", "pattern", "ping", "placeholder", "poster",
        "preload", "radiogroup", "readonly", "referrerpolicy", "rel", "required", "reversed",
        "rows", "rowspan", "sandbox", "scope", "scoped", "selected", "shape", "size", "sizes",
        "slot", "span", "spellcheck", "src", "srcdoc", "srclang", "srcset", "start", "step",
        "style", "summary", "tabindex", "target", "title", "translate", "type", "usemap", "value",
        "width", "wrap",

        // alpine.js attributes
        "@click", "@keyup.enter", "@keyup.shift.enter", "x-bind", "x-cloak", "x-data", "x-effect",
        "x-for", "x-html", "x-id", "x-if", "x-ignore", "x-init", "x-model", "x-modelable", "x-on",
        "x-ref", "x-show", "x-teleport", "x-text", "x-transition",
    ];

    #[rustfmt::skip]
    pub(crate) static HTML_ATTRIBUTE_VALUES_RAW: &[&str] = &[
        "true", "false", "100",
    ];

    #[rustfmt::skip]
    pub(crate) static HTML_ATTRIBUTE_VALUES_QUOTED: &[&str] = &[
        r#""open""#, r#""UTF-8""#, r#""/style.css""#,
        r#""width=device-width, initial-scale=1.0""#,

        // alpine.js
        r#""{ open: false }""#, r#""alert('Hello World!')""#,
        r#""{ open: false, toggle() { this.open = ! this.open } }""#,
    ];
}
