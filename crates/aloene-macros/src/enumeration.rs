pub fn derive(enum_name: syn::Ident, data_enum: syn::DataEnum) -> proc_macro2::TokenStream {
    let fn_variant_names = enum_variant_name(&enum_name, &data_enum);

    let variants = data_enum
        .variants
        .iter()
        .map(|variant| variant.ident.to_string());

    let de_match = data_enum.variants.iter().map(de_variant);
    let se_match = data_enum.variants.iter().map(se_variant);

    quote::quote! {
        impl ::aloene::Aloene for #enum_name {
            fn deserialize<R: ::std::io::Read>(reader: &mut R) -> ::std::result::Result<Self, ::aloene::Error> {
                ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::VARIANT)?;

                ::aloene::io::assert_byte(reader, ::aloene::bytes::Value::STRING)?;

                let variant = ::aloene::io::read_string(reader)?;

                match variant.as_str() {
                    #( #de_match )*
                    _ => Err(::aloene::Error::UnknownVariant { expected: &[ #( #variants ),* ], got: variant }),
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

fn de_variant(variant: &syn::Variant) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    let handler = match &variant.fields {
        syn::Fields::Named(_named) => {
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

            syn_err!(
                variant,
                "Aloene does not yet support enums with struct like variants"
            )
        }
        syn::Fields::Unnamed(_unnamed) => {
            // let fields = unnamed.unnamed.iter().map(|field| {
            //     quote::quote! {}
            // });

            // quote::quote! {
            //     ::aloene::io::assert_byte(reader, ::aloene::bytes::Container::ARRAY)?;

            //     #( #fields )*
            // }

            syn_err!(
                variant,
                "Aloene does not yet support enums with tuple like variants"
            )
        }
        syn::Fields::Unit => {
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

fn se_variant(variant: &syn::Variant) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    let handler = match &variant.fields {
        syn::Fields::Named(_named) => syn_err!(
            variant,
            "Aloene does not yet support enums with struct like variants"
        ),
        syn::Fields::Unnamed(_unnamed) => syn_err!(
            variant,
            "Aloene does not yet support enums with tuple like variants"
        ),
        syn::Fields::Unit => {
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

fn enum_variant_name(ident: &syn::Ident, data_enum: &syn::DataEnum) -> proc_macro2::TokenStream {
    let variant_idents = data_enum.variants.iter().map(|variant| &variant.ident);
    let variant_kind_idents = data_enum.variants.iter().map(|variant| &variant.ident);

    let variant_kind = data_enum
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Named(_) => quote::quote! { { .. } },
            syn::Fields::Unnamed(unnamed) => {
                let fields = unnamed.unnamed.len();

                let blanks = (0..fields).map(|_| quote::quote! { _ });

                quote::quote! { ( #( #blanks ),* ) }
            }
            syn::Fields::Unit => quote::quote! {},
        });

    quote::quote! {
        fn get_name(enumeration: &#ident) -> &'static str {
            match enumeration {
                #( #ident::#variant_kind_idents #variant_kind => stringify!(#variant_idents), )*
            }
        }
    }
}
