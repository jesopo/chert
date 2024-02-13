use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, Data::Struct, DataStruct, DeriveInput, Fields::Named,
    FieldsNamed, Token, Type,
};

mod kw {
    syn::custom_keyword!(as_ref);
}

enum ChertAttribute {
    AsRef {
        _as_ref: kw::as_ref,
        _equals: Token![=],
        as_type: Type,
    },
}

impl Parse for ChertAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::as_ref) {
            Ok(Self::AsRef {
                _as_ref: input.parse()?,
                _equals: input.parse()?,
                as_type: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[proc_macro_derive(Variables, attributes(chert))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: struct_name,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let Struct(DataStruct {
        fields: Named(FieldsNamed {
            named: ref named_fields,
            ..
        }),
        ..
    }) = data
    else {
        panic!("must be a struct with named fields");
    };

    let mut fields = Vec::new();
    let mut accessor_functions = Vec::new();

    for field in named_fields.iter() {
        let Some(field_name) = &field.ident else {
            continue;
        };
        let mut field_type = field.ty.clone();
        let mut use_as_ref = false;

        for attr in &field.attrs {
            if attr.path().is_ident("chert") {
                let Ok(chert_attr) = attr.parse_args::<ChertAttribute>() else {
                    panic!("Invalid chert attribute: {}", attr.to_token_stream())
                };

                let ChertAttribute::AsRef { as_type, .. } = chert_attr;
                field_type = as_type;
                use_as_ref = true;
            }
        }

        let accessor_name = Ident::new(
            &format!("_chert_get_{}", field_name.to_string().to_ascii_lowercase()),
            field_name.span(),
        );

        let ident_str = field_name.to_string();

        fields.push(quote! {
            (#ident_str, <#field_type as chert::variables::VariableType>::from_field(Self::#accessor_name))
        });

        if use_as_ref {
            accessor_functions.push(quote! {
                #[allow(non_snake_case)]
                fn #accessor_name(object: &#struct_name) -> &<#field_type as chert::variables::VariableType>::AccessedAs {
                    use std::convert::AsRef;
                    object.#field_name.as_ref()
                }
            });
        } else {
            accessor_functions.push(quote! {
                #[allow(non_snake_case)]
                fn #accessor_name(object: &#struct_name) -> &<#field_type as chert::variables::VariableType>::AccessedAs {
                    &object.#field_name
                }
            });
        }
    }

    quote! {
        impl #struct_name {
            #(#accessor_functions)*
        }

        impl chert::variables::Variables for #struct_name {
            fn variables() -> std::collections::HashMap<&'static str, chert::variables::Variable<Self>> {
                std::collections::HashMap::from([#(#fields),*])
            }
        }
    }
    .into()
}
