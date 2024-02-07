use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Data::Struct, DataStruct, DeriveInput, Fields::Named, FieldsNamed};

#[proc_macro_derive(ChertStruct)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let Struct(DataStruct {
        fields: Named(FieldsNamed { ref named, .. }),
        ..
    }) = data
    else {
        panic!("must be a struct with named fields");
    };

    let mut fields = Vec::new();
    let mut accessor_functions = Vec::new();

    for (i, t) in named
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| (i, &f.ty)))
    {
        let accessor_name = Ident::new(
            &format!("_chert_get_{}", i.to_string().to_ascii_lowercase()),
            i.span(),
        );

        let ident_str = i.to_string();

        fields.push(quote! {
            (#ident_str, <#t as chert::ChertFieldType>::from_field(Self::#accessor_name))
        });

        accessor_functions.push(quote! {
            #[allow(non_snake_case)]
            fn #accessor_name(object: &#ident) -> &#t {
                &object.#i
            }
        });
    }

    quote! {
        impl #ident {
            #(#accessor_functions)*
        }

        impl chert::ChertStructTrait for #ident {
            fn fields() -> std::collections::HashMap<String, (usize, chert::ChertField<Self>)> {
                use std::collections::HashMap;
                use chert::ChertField;

                let mut field_counts: HashMap<u8, usize> = HashMap::new();
                let mut indexed_fields: HashMap<String, (usize, ChertField<Self>)> = HashMap::new();
                let unindexed_fields: HashMap<&'static str, ChertField<Self>> = HashMap::from([#(#fields),*]);

                for (name, field) in unindexed_fields.into_iter() {
                    let type_key = field.type_key();
                    if let Some(i) = field_counts.get(&type_key) {
                        field_counts.insert(type_key, i + 1);
                    } else {
                        field_counts.insert(type_key, 0);
                    }
                    indexed_fields.insert(name.to_owned(), (field_counts[&type_key], field));
                }

                indexed_fields
            }
        }
    }
    .into()
}
