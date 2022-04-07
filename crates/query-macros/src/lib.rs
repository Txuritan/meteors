extern crate proc_macro;

use std::collections::HashMap;

use syn::{Expr, ItemStatic, Lit};

macro_rules! syn_err {
    ($to:expr, $ms:expr) => {
        proc_macro::TokenStream::from(syn::Error::new_spanned($to, $ms).to_compile_error())
    };
}

#[proc_macro_attribute]
pub fn selector(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut dec = syn::parse_macro_input!(input as ItemStatic);

    let raw_selector = {
        let expr_lit = if let Expr::Lit(expr_lit) = &*dec.expr {
            expr_lit
        } else {
            return syn_err!(dec.expr, "Selector can only be a string literal");
        };

        let lit_str = if let Lit::Str(lit_str) = &expr_lit.lit {
            lit_str
        } else {
            return syn_err!(dec.expr, "Selector can only be a string literal");
        };

        Selector::from(lit_str.value())
    };

    let selector_struct_ident = quote::format_ident!("Selector_{}", dec.ident);
    let selector_module_ident = quote::format_ident!("selector_{}", dec.ident);

    let selector_member_stmt = raw_selector
        .matchers
        .iter()
        .enumerate()
        .map(|(i, matcher)| {
            let member = quote::format_ident!("matcher_{}", i);
            let matcher_stmt = matcher_to_stmt_tokens(matcher);

            quote::quote! { pub #member: #matcher_stmt }
        });

    let select_match_stmt = raw_selector.matchers.iter().enumerate().map(|(i, _)| {
        let member = quote::format_ident!("matcher_{}", i);

        quote::quote! {
            {
                if self. #member .direct_match {
                    direct_match = true;

                    elements = Self::elements(elements);
                } else {
                    elements = ::query::compile_time::find_nodes(&self. #member, &elements, direct_match);
                    direct_match = false;
                }
            }
        }
    });

    let selector_module = quote::quote! {
        #[allow(non_snake_case)]
        pub(self) mod #selector_module_ident {
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            pub struct #selector_struct_ident {
                #( #selector_member_stmt , )*
            }

            impl ::query::Selector for #selector_struct_ident {
                fn find<'input>(&self, elements: &[::query::Node<'input>]) -> Vec<::query::Node<'input>> {
                    let mut elements: Vec<_> = elements.to_vec();
                    let mut direct_match = false;

                    #( #select_match_stmt )*

                    elements.to_vec()
                }
            }
        }
    };

    let selector_member_expr = raw_selector
        .matchers
        .iter()
        .enumerate()
        .map(|(i, matcher)| {
            let member = quote::format_ident!("matcher_{}", i);
            let matcher_expr = matcher_to_expr_tokens(matcher);

            quote::quote! { #member: #matcher_expr }
        });

    let selector_expr = quote::quote! {
        #selector_module_ident :: #selector_struct_ident {
            #( #selector_member_expr , )*
        }
    };

    dec.ty = Box::new(syn::parse_quote! {
        #selector_module_ident :: #selector_struct_ident
    });
    dec.expr = Box::new(syn::parse_quote! {
        #selector_expr
    });

    let stream = quote::quote! {
        #selector_module
        #dec
    };

    stream.into()
}

fn matcher_to_stmt_tokens(matcher: &Matcher) -> proc_macro2::TokenStream {
    let tags = matcher.tag.len();
    let classes = matcher.class.len();
    let ids = matcher.id.len();
    let attributes = matcher.attribute.len();

    quote::quote! {
        ::query::compile_time::StaticMatcher<#tags, #classes, #ids, #attributes>
    }
}

fn matcher_to_expr_tokens(matcher: &Matcher) -> proc_macro2::TokenStream {
    let tags = matcher.tag.iter().map(|i| quote::quote! { #i });
    let classes = matcher.class.iter().map(|i| quote::quote! { #i });
    let ids = matcher.id.iter().map(|i| quote::quote! { #i });

    let attributes = matcher.attribute.iter().map(|(key, value)| {
        let value = match value {
            AttributeSpec::Present => {
                quote::quote! { ::query::compile_time::StaticAttributeSpec::Present }
            }
            AttributeSpec::Exact(t) => {
                quote::quote! { ::query::compile_time::StaticAttributeSpec::Exact(#t) }
            }
            AttributeSpec::Starts(t) => {
                quote::quote! { ::query::compile_time::StaticAttributeSpec::Starts(#t) }
            }
            AttributeSpec::Ends(t) => {
                quote::quote! { ::query::compile_time::StaticAttributeSpec::Ends(#t) }
            }
            AttributeSpec::Contains(t) => {
                quote::quote! { ::query::compile_time::StaticAttributeSpec::Contains(#t) }
            }
        };

        quote::quote! { ( #key, #value ) }
    });

    let direct_match = matcher.direct_match;

    quote::quote! {
        ::query::compile_time::StaticMatcher {
            tag: [ #( #tags , )* ],
            class: [ #( #classes , )* ],
            id: [ #( #ids , )* ],
            attribute: [ #( #attributes , )* ],
            direct_match: #direct_match,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Selector {
    matchers: Vec<Matcher>,
}

impl From<&str> for Selector {
    fn from(input: &str) -> Self {
        let matchers: Vec<_> = input.split_whitespace().map(Matcher::from).collect();

        Self { matchers }
    }
}

impl From<String> for Selector {
    fn from(input: String) -> Self {
        let matchers: Vec<_> = input.split_whitespace().map(Matcher::from).collect();

        Self { matchers }
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

#[derive(Debug, PartialEq, Clone)]
enum AttributeSpec {
    Present,
    Exact(String),
    Starts(String),
    Ends(String),
    Contains(String),
}
