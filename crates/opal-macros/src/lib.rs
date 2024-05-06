mod parser;

use std::str::FromStr as _;

use parser::Stage4;

#[proc_macro_derive(Template, attributes(template))]
pub fn template_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let decl = match venial::parse_item(proc_macro2::TokenStream::from(input)) {
        Ok(decl) => decl,
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };

    let (name, generic_params, where_clause, inline_generic_args, attributes) = match &decl {
        venial::Item::Struct(decl) => (
            &decl.name,
            &decl.generic_params,
            &decl.where_clause,
            decl.get_inline_generic_args(),
            &decl.attributes,
        ),
        venial::Item::Enum(decl) => (
            &decl.name,
            &decl.generic_params,
            &decl.where_clause,
            decl.get_inline_generic_args(),
            &decl.attributes,
        ),
        _ => {
            return venial::Error::new_at_tokens(
                decl,
                "Aloene can only be used on `structs`s and `enum`'s",
            )
            .to_compile_error()
            .into()
        }
    };

    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => std::path::PathBuf::from(dir),
        Err(err) => {
            return venial::Error::new_at_tokens(decl, err)
                .to_compile_error()
                .into()
        }
    };

    let path = attributes
        .iter()
        .find(|attr| {
            attr.path
                .first()
                .map(|tree| {
                    if let proc_macro2::TokenTree::Ident(ident) = tree {
                        ident == "template"
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        })
        .and_then(|attr| match &attr.value {
            venial::AttributeValue::Group(span, stream) => {
                Some(Pair::parse_stream(span.span, stream))
            }
            _ => None,
        })
        .transpose();

    let path = match path {
        Ok(path) => path,
        Err(err) => {
            return venial::Error::new_at_tokens(decl, err)
                .to_compile_error()
                .into()
        }
    };

    let path = match path {
        Some(path) => path,
        None => {
            return venial::Error::new_at_tokens(decl, "missing path attribute")
                .to_compile_error()
                .into()
        }
    };

    let temp_template_path = root_dir.join("templates").join(path.value);

    let template_path = match temp_template_path.canonicalize() {
        Ok(path) => path,
        Err(err) => {
            return venial::Error::new_at_tokens(
                decl,
                format!("`{}`: {}", temp_template_path.display(), err),
            )
            .to_compile_error()
            .into()
        }
    };

    let content = match std::fs::read_to_string(&template_path) {
        Ok(content) => content,
        Err(err) => {
            return venial::Error::new_at_tokens(decl, err)
                .to_compile_error()
                .into()
        }
    };

    let tokens = parser::parse(&content);

    let size_hint = match write_size_hint(&tokens) {
        Ok(size_hint) => size_hint,
        Err(err) => {
            return venial::Error::new_at_tokens(decl, err)
                .to_compile_error()
                .into()
        }
    };

    let render = match write_render(&tokens) {
        Ok(size_hint) => size_hint,
        Err(err) => {
            return venial::Error::new_at_tokens(decl, err)
                .to_compile_error()
                .into()
        }
    };

    let source_ident = quote::format_ident!("{}_SOURCE", name.to_string().to_uppercase());
    let source_path = template_path.to_str().unwrap();

    let tokens = quote::quote! {
        #[allow(dead_code)]
        const #source_ident: &str = include_str!(#source_path);

        impl #generic_params ::opal::Template for #name #inline_generic_args #where_clause {
            fn size_hint(&self) -> usize {
                let mut hint = 0;

                #size_hint

                hint
            }

            fn render<W>(&self, writer: &mut W) -> ::std::result::Result<(), W::Error>
            where
                W: ::opal::io::Write,
            {
                #[allow(dead_code)]
                use ::opal::{{Template as _, io::{{vfmt, Write as _}}}};

                #render

                Ok(())
            }
        }
    };

    // panic!("{}", tokens.to_string());

    proc_macro::TokenStream::from(tokens)
}

struct Pair {
    _key: proc_macro2::Ident,
    _eq_token: proc_macro2::Punct,
    value: String,
}

impl Pair {
    fn parse_stream(
        span: proc_macro2::Span,
        stream: &[proc_macro2::TokenTree],
    ) -> Result<Self, venial::Error> {
        let mut stream_iter = stream.iter();

        let key = stream_iter
            .next()
            .and_then(|tree| {
                if let proc_macro2::TokenTree::Ident(ident) = tree {
                    Some(ident.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| venial::Error::new_at_span(span, "no key ident found"))?;

        let token = stream_iter
            .next()
            .and_then(|tree| {
                if let proc_macro2::TokenTree::Punct(punct) = tree {
                    Some(punct.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| venial::Error::new_at_span(span, "no `=` found"))?;

        let value = stream_iter
            .next()
            .and_then(|tree| {
                if let proc_macro2::TokenTree::Literal(literal) = tree {
                    Some(literal)
                } else {
                    None
                }
            })
            .map(|literal| {
                literal
                    .to_string()
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .to_string()
            })
            .ok_or_else(|| venial::Error::new_at_span(span, "no path literal found"))?;

        Ok(Self {
            _key: key,
            _eq_token: token,
            value,
        })
    }
}

// impl syn::parse::Parse for Pair {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         Ok(Pair {
//             _key: input.parse()?,
//             _eq_token: input.parse()?,
//             value: input.parse()?,
//         })
//     }
// }

fn write_size_hint(tokens: &[Stage4]) -> Result<proc_macro2::TokenStream, proc_macro2::LexError> {
    let mut stream = quote::quote! {};

    for token in tokens {
        match token {
            Stage4::Expr(expr) => {
                if expr.trim() == "count" {
                    continue;
                }

                if !(expr.contains('+') || expr.contains('-') || expr.contains("len")) {
                    let v = proc_macro2::TokenStream::from_str(expr.trim())?;

                    stream = quote::quote! {
                        #stream

                        hint += #v.len();
                    };
                }
            }
            Stage4::ExprAssign(expr) => {
                let v = proc_macro2::TokenStream::from_str(expr.trim())?;

                stream = quote::quote! {
                    #stream

                    #v
                };
            }
            Stage4::ExprRender(expr) => {
                let v = proc_macro2::TokenStream::from_str(
                    expr.trim().trim_end_matches(".render(writer)"),
                )?;

                stream = quote::quote! {
                    #stream

                    hint += &#v.size_hint();
                };
            }
            Stage4::If(cond, if_tokens, else_tokens) => {
                let inner_stream = write_size_hint(if_tokens)?;

                let v = proc_macro2::TokenStream::from_str(cond)?;
                stream = quote::quote! {
                    #stream

                    #v {
                        #inner_stream
                    }
                };

                if let Some(else_tokens) = else_tokens {
                    let inner_stream = write_size_hint(else_tokens)?;

                    stream = quote::quote! {
                        #stream

                        else {
                            #inner_stream
                        }
                    };
                }
            }
            Stage4::For(cond, tokens) => {
                let inner_tokens = write_size_hint(tokens)?;

                let v = proc_macro2::TokenStream::from_str(cond)?;

                stream = quote::quote! {
                    #stream

                    #v {
                        #inner_tokens
                    }
                };
            }
            Stage4::Other(other) => {
                let v = other.len();

                stream = quote::quote! {
                    #stream

                    hint += #v;
                };
            }
        }
    }

    Ok(stream)
}

fn write_render(tokens: &[Stage4]) -> Result<proc_macro2::TokenStream, proc_macro2::LexError> {
    let mut stream = quote::quote! {};

    for token in tokens {
        match token {
            Stage4::Expr(expr) => {
                let expr = proc_macro2::TokenStream::from_str(
                    &expr.replace('{', "{{").replace('}', "}}"),
                )?;

                stream = quote::quote! {
                    #stream

                    ::opal::io::write!(writer, "{}", #expr)?;
                };
            }
            Stage4::ExprAssign(expr) => {
                let expr = proc_macro2::TokenStream::from_str(expr.trim())?;

                stream = quote::quote! {
                    #stream

                    #expr
                };
            }
            Stage4::ExprRender(expr) => {
                let expr = proc_macro2::TokenStream::from_str(expr.trim())?;

                stream = quote::quote! {
                    #stream

                    #expr?;
                };
            }
            Stage4::If(cond, if_tokens, else_tokens) => {
                let inner_stream = write_render(if_tokens)?;

                let cond = proc_macro2::TokenStream::from_str(cond)?;

                stream = quote::quote! {
                    #stream

                    #cond {
                        #inner_stream
                    }
                };

                if let Some(else_tokens) = else_tokens {
                    let inner_stream = write_render(else_tokens)?;

                    stream = quote::quote! {
                        #stream

                        else {
                            #inner_stream
                        }
                    };
                }
            }
            Stage4::For(cond, tokens) => {
                let inner_stream = write_render(tokens)?;

                let cond = proc_macro2::TokenStream::from_str(cond)?;

                stream = quote::quote! {
                    #stream

                    #cond {
                        #inner_stream
                    }
                };
            }
            Stage4::Other(other) => {
                let other = other.replace('{', "{{").replace('}', "}}");

                stream = quote::quote! {
                    #stream

                    ::opal::io::write!(writer, #other)?;
                };
            }
        }
    }

    Ok(stream)
}
