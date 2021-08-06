pub fn derive(
    struct_name: syn::Ident,
    data_struct: syn::DataStruct,
    generics: syn::Generics,
) -> proc_macro2::TokenStream {
    let field_iter = data_struct.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        quote::quote! { #ident }
    });

    let de_iter = data_struct.fields.iter().map(de_field);
    let se_iter = data_struct.fields.iter().map(se_field);

    let syn::Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    } = generics;

    quote::quote! {
        impl #lt_token #params #gt_token ::aloene::Aloene for #struct_name #lt_token #params #gt_token #where_clause {
            fn deserialize<R: ::std::io::Read>(reader: &mut R) -> ::std::result::Result<Self, ::aloene::Error> {
                ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::STRUCT)?;

                #( #de_iter )*

                Ok(Self { #( #field_iter , )* })
            }

            fn serialize<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::result::Result<(), ::aloene::Error> {
                ::aloene::io::write_u8(writer, ::aloene::bytes::Container::STRUCT)?;

                #( #se_iter )*

                Ok(())
            }
        }
    }
}

fn field_help(field: &syn::Field) -> Result<(&syn::Ident, &syn::Ident), proc_macro2::TokenStream> {
    let field_name = match &field.ident {
        Some(ident) => ident,
        None => {
            return Err(syn_err!(field, "Invalid field"));
        }
    };

    let field_path = match &field.ty {
        syn::Type::Path(field_path) => field_path,
        _ => {
            return Err(syn_err!(field, "Invalid field"));
        }
    };

    let path_segments = &field_path.path.segments;

    let first_segment = match path_segments.iter().next() {
        Some(first_segment) => first_segment,
        None => {
            return Err(syn_err!(path_segments, "Invalid type"));
        }
    };

    let ident = &first_segment.ident;

    Ok((field_name, ident))
}

#[rustfmt::skip]
static BUILTIN_TYPES: [&str; 12] = [
    "f32", "f64",
    "i8", "i16", "i32", "i64", "isize",
    "u8", "u16", "u32", "u64", "usize",
];

pub fn de_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let (field_name, ident) = match field_help(field) {
        Ok(pair) => pair,
        Err(err) => return err,
    };

    let typ = BUILTIN_TYPES.iter().find(|typ| ident == **typ);

    if let Some(typ) = typ {
        let function = quote::format_ident!("read_{}", typ);

        quote::quote! {
            let #field_name = ::aloene::io::structure::#function(reader)?;
        }
    } else {
        quote::quote! {
            ::aloene::io::assert_byte(reader, ::aloene::bytes::Value::STRING)?;

            let _field = ::aloene::io::read_string(reader)?;

            ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::VALUE)?;
            let #field_name = ::aloene::Aloene::deserialize(reader)?;
        }
    }
}

fn se_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let (field_name, ident) = match field_help(field) {
        Ok(pair) => pair,
        Err(err) => return err,
    };

    let typ = BUILTIN_TYPES.iter().find(|typ| ident == **typ);

    if let Some(typ) = typ {
        let function = quote::format_ident!("write_{}", typ);

        quote::quote! {
            ::aloene::io::structure::#function(writer, stringify!(#field_name), self . #field_name)?;
        }
    } else {
        quote::quote! {
            ::aloene::io::write_u8(writer, ::aloene::bytes::Value::STRING)?;

            ::aloene::io::write_string(writer, stringify!(#field_name))?;

            ::aloene::io::write_u8(writer, ::aloene::bytes::Container::VALUE)?;
            ::aloene::Aloene::serialize(&self . #field_name, writer)?;
        }
    }
}
