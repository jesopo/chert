use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, Data::Struct, DataStruct, DeriveInput, Fields::Named, FieldsNamed, Type,
};

fn to_chert_field(ident: &Ident, ty: &Type) -> TokenStream2 {
    let ident_str = ident.to_string();

    if let Type::Path(type_path) = ty {
        quote! {
            (#ident_str, <chert::ChertField::<Self> as From<Box<dyn Fn(&Self) -> &#type_path>>>::from(Box::new(|o| &o.#ident)))
        }
    } else {
        unreachable!();
    }
}

#[proc_macro_derive(ChertStruct)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let fields = if let Struct(DataStruct {
        fields: Named(FieldsNamed { ref named, .. }),
        ..
    }) = data
    {
        named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| (i, &f.ty)))
            .map(|(i, t)| to_chert_field(i, t))
            .collect::<Vec<_>>()
    } else {
        panic!("must be a struct with named fields");
    };

    quote! {
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
