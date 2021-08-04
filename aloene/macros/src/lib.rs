macro_rules! syn_err {
    ($spanned:expr, $msg:expr) => {
        syn::Error::new_spanned($spanned, $msg).to_compile_error()
    };
}

mod enumeration;
mod structure;

#[proc_macro_derive(Aloene, attributes(aloene))]
pub fn aloene_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive = syn::parse_macro_input!(input as syn::DeriveInput);

    let stream = match derive.data {
        syn::Data::Enum(data_enum) => enumeration::derive(derive.ident, data_enum),
        syn::Data::Struct(data_struct) => {
            structure::derive(derive.ident, data_struct, derive.generics)
        }
        _ => {
            syn_err!(derive, "Aloene can not be used on `union`s")
        }
    };

    if false {
        eprintln!("{}", stream.to_string())
    }

    proc_macro::TokenStream::from(stream)
}
