macro_rules! err {
    ($spanned:expr, $msg:expr) => {
        venial::Error::new_at_tokens($spanned, $msg).to_compile_error()
    };
}

mod enumeration;
mod structure;

#[proc_macro_derive(Aloene, attributes(aloene))]
pub fn aloene_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let decl = match venial::parse_item(proc_macro2::TokenStream::from(input)) {
        Ok(decl) => decl,
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };

    let stream = match decl {
        venial::Item::Struct(decl) => structure::derive(decl),
        venial::Item::Enum(decl) => enumeration::derive(decl.name, decl.variants),
        _ => err!(decl, "Aloene can only be used on `structs`s or `enum`s"),
    };

    if false {
        eprintln!("{}", stream)
    }

    proc_macro::TokenStream::from(stream)
}
