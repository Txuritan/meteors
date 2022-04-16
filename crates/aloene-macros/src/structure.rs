pub fn derive(decl: venial::Struct) -> proc_macro2::TokenStream {
    let inline_generic_params = decl.get_inline_generic_args();

    let venial::Struct {
        name,
        generic_params,
        where_clause,
        fields,
        ..
    } = &decl;

    let field_iter = match &fields {
        venial::StructFields::Named(named) => named
            .fields
            .iter()
            .map(|field| {
                let ident = &field.0.name;

                quote::quote! { #ident }
            })
            .collect::<Vec<_>>(),
        _ => return err!(fields, "aloene does not support unit or tuple structs."),
    };

    let de_iter = match &fields {
        venial::StructFields::Named(named) => named.fields.iter().map(|(field, _)| de_field(field)),
        _ => return err!(fields, "aloene does not support unit or tuple structs."),
    };
    let se_iter = match &fields {
        venial::StructFields::Named(named) => named.fields.iter().map(|(field, _)| se_field(field)),
        _ => return err!(fields, "aloene does not support unit or tuple structs."),
    };

    quote::quote! {
        impl #generic_params ::aloene::Aloene for #name #inline_generic_params #where_clause {
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

fn field_help(
    field: &venial::NamedField,
) -> Result<(&proc_macro2::Ident, &proc_macro2::Ident), proc_macro2::TokenStream> {
    let field_name = &field.name;

    let ident = match field.ty.tokens.first() {
        Some(tree) => match tree {
            proc_macro2::TokenTree::Ident(name) => name,
            _ => return Err(err!(field, "aloene structs field type mut be an ident.")),
        },
        None => return Err(err!(field, "aloene structs field require a type.")),
    };

    Ok((field_name, ident))
}

#[rustfmt::skip]
static BUILTIN_TYPES: [&str; 12] = [
    "f32", "f64",
    "i8", "i16", "i32", "i64", "isize",
    "u8", "u16", "u32", "u64", "usize",
];

pub fn de_field(field: &venial::NamedField) -> proc_macro2::TokenStream {
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

fn se_field(field: &venial::NamedField) -> proc_macro2::TokenStream {
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
