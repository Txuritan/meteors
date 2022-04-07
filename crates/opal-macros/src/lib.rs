mod parser;

use std::str::FromStr as _;

use parser::Stage4;

#[proc_macro_derive(Template, attributes(template))]
pub fn template_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive = syn::parse_macro_input!(input as syn::DeriveInput);

    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => std::path::PathBuf::from(dir),
        Err(err) => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, err).to_compile_error(),
            )
        }
    };

    let path = derive
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("template"))
        .map(|attr| attr.parse_args::<Pair>())
        .transpose();

    let path = match path {
        Ok(path) => path,
        Err(err) => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, err).to_compile_error(),
            )
        }
    };

    let path = match path {
        Some(path) => path,
        None => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, "missing path attribute").to_compile_error(),
            )
        }
    };

    let template_path = match root_dir
        .join("templates")
        .join(&path.value.value())
        .canonicalize()
    {
        Ok(path) => path,
        Err(err) => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, err).to_compile_error(),
            )
        }
    };

    let content = match std::fs::read_to_string(&template_path) {
        Ok(content) => content,
        Err(err) => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, err).to_compile_error(),
            )
        }
    };

    let syn::DeriveInput {
        ident,
        generics:
            syn::Generics {
                lt_token,
                params,
                gt_token,
                where_clause,
            },
        ..
    } = &derive;

    let tokens = parser::parse(&content);

    let size_hint = match write_size_hint(&tokens) {
        Ok(size_hint) => size_hint,
        Err(err) => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, err).to_compile_error(),
            )
        }
    };

    let render = match write_render(&tokens) {
        Ok(size_hint) => size_hint,
        Err(err) => {
            return proc_macro::TokenStream::from(
                syn::Error::new_spanned(derive, err).to_compile_error(),
            )
        }
    };

    let source_ident = quote::format_ident!("{}_SOURCE", ident.to_string().to_uppercase());
    let source_path = template_path.to_str().unwrap();

    let tokens = quote::quote! {
        #[allow(dead_code)]
        const #source_ident: &str = include_str!(#source_path);

        impl #lt_token #params #gt_token ::opal::Template for #ident #lt_token #params #gt_token #where_clause {
            fn size_hint(&self) -> usize {
                let mut hint = 0;

                #size_hint

                hint
            }

            fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>
            where
                W: ::std::io::Write,
            {
                use {{::opal::Template as _, ::std::io::Write as _}};

                #render

                Ok(())
            }
        }
    };

    // panic!("{}", tokens.to_string());

    proc_macro::TokenStream::from(tokens)
}

struct Pair {
    _key: syn::Ident,
    _eq_token: syn::Token![=],
    value: syn::LitStr,
}

impl syn::parse::Parse for Pair {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Pair {
            _key: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

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

                    write!(writer, "{}", #expr)?;
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

                    write!(writer, #other)?;
                };
            }
        }
    }

    Ok(stream)
}
