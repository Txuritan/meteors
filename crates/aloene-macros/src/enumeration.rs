pub fn derive(
    enum_name: proc_macro2::Ident,
    variants: venial::Punctuated<venial::EnumVariant>,
) -> proc_macro2::TokenStream {
    let fn_variant_names = enum_variant_name(&enum_name, &variants);

    let variants_names = variants.iter().map(|(variant, _)| variant.name.to_string());

    let de_match = variants.iter().map(|(variant, _)| de_variant(variant));
    let se_match = variants.iter().map(|(variant, _)| se_variant(variant));

    quote::quote! {
        impl ::aloene::Aloene for #enum_name {
            fn deserialize<R: ::std::io::Read>(reader: &mut R) -> ::std::result::Result<Self, ::aloene::Error> {
                ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::VARIANT)?;

                ::aloene::io::assert_byte(reader, ::aloene::bytes::Value::STRING)?;

                let variant = ::aloene::io::read_string(reader)?;

                match variant.as_str() {
                    #( #de_match )*
                    _ => Err(::aloene::Error::UnknownVariant { expected: &[ #( #variants_names ),* ], got: variant }),
                }
            }

            fn serialize<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::result::Result<(), ::aloene::Error> {
                #fn_variant_names

                ::aloene::io::write_u8(writer, ::aloene::bytes::Container::VARIANT)?;

                match &self {
                    #( Self :: #se_match )*
                }

                Ok(())
            }
        }
    }
}

fn de_variant(variant: &venial::EnumVariant) -> proc_macro2::TokenStream {
    let variant_ident = &variant.name;

    let handler = match &variant.contents {
        venial::StructFields::Named(_named) => {
            // let field_idents = named.named.iter().map(|field| {
            //     let ident = field.ident.as_ref().unwrap();

            //     quote::quote! { #ident }
            // });

            // let fields = named.named.iter().map(super::structure::de_field);

            // quote::quote! {
            //     ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::STRUCT)?;

            //     #( #fields )*

            //     Ok(Self::#variant_ident { #( #field_idents , )* })
            // }

            err!(
                variant,
                "Aloene does not yet support enums with struct like variants"
            )
        }
        venial::StructFields::Tuple(_unnamed) => {
            // let fields = unnamed.unnamed.iter().map(|field| {
            //     quote::quote! {}
            // });

            // quote::quote! {
            //     ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::ARRAY)?;

            //     #( #fields )*
            // }

            err!(
                variant,
                "Aloene does not yet support enums with tuple like variants"
            )
        }
        venial::StructFields::Unit => {
            quote::quote! {
                ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::UNIT)?;

                Ok(Self::#variant_ident)
            }
        }
    };

    quote::quote! {
        stringify!(#variant_ident) => { #handler },
    }
}

fn se_variant(variant: &venial::EnumVariant) -> proc_macro2::TokenStream {
    let variant_ident = &variant.name;

    let handler = match &variant.contents {
        venial::StructFields::Named(_named) => err!(
            variant,
            "Aloene does not yet support enums with struct like variants"
        ),
        venial::StructFields::Tuple(_unnamed) => err!(
            variant,
            "Aloene does not yet support enums with tuple like variants"
        ),
        venial::StructFields::Unit => {
            quote::quote! {
                ::aloene::io::write_u8(writer, ::aloene::bytes::Value::STRING)?;

                ::aloene::io::write_string(writer, stringify!(#variant_ident))?;

                ::aloene::io::write_u8(writer, ::aloene::bytes::Container::UNIT)?;
            }
        }
    };

    quote::quote! {
        #variant_ident => { #handler },
    }
}

fn enum_variant_name(
    ident: &proc_macro2::Ident,
    variants: &venial::Punctuated<venial::EnumVariant>,
) -> proc_macro2::TokenStream {
    let variant_idents = variants.iter().map(|(variant, _)| &variant.name);
    let variant_kind_idents = variants.iter().map(|(variant, _)| &variant.name);

    let variant_kind = variants.iter().map(|(variant, _)| match &variant.contents {
        venial::StructFields::Named(_) => quote::quote! { { .. } },
        venial::StructFields::Tuple(unnamed) => {
            let fields = unnamed.fields.len();

            let blanks = (0..fields).map(|_| quote::quote! { _ });

            quote::quote! { ( #( #blanks ),* ) }
        }
        venial::StructFields::Unit => quote::quote! {},
    });

    quote::quote! {
        fn get_name(enumeration: &#ident) -> &'static str {
            match enumeration {
                #( #ident::#variant_kind_idents #variant_kind => stringify!(#variant_idents), )*
            }
        }
    }
}
