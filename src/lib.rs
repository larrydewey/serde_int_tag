use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, parse_macro_input};

#[proc_macro_derive(IntTag, attributes(tag))]
pub fn derive_int_tag(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let serialize_count = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let is_option = matches!(
            &field.ty,
            syn::Type::Path(syn::TypePath { path, .. })
            if path.segments.iter().any(|seg| seg.ident == "Option")
        );

        if is_option {
            quote! {
                if self.#field_name.is_some() {
                    count += 1;
                }
            }
        } else {
            quote! {
                count += 1;
            }
        }
    });

    let serialize_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let tag = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("tag"))
            .and_then(|attr| {
                if let Meta::List(meta_list) = &attr.meta {
                    meta_list.parse_args::<syn::LitInt>().ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("Field `{field_name}` must have a #[tag(N)] attribute"));

        let is_option = matches!(
            &field.ty,
            syn::Type::Path(syn::TypePath { path, .. })
            if path.segments.iter().any(|seg| seg.ident == "Option")
        );

        if is_option {
            quote! {
                if let Some(ref value) = self.#field_name {
                    map.serialize_entry(&(#tag as u64), value)?;
                }
            }
        } else {
            quote! {
                map.serialize_entry(&(#tag as u64), &self.#field_name)?;
            }
        }
    });

    let deserialize_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let tag = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("tag"))
            .and_then(|attr| {
                if let Meta::List(meta_list) = &attr.meta {
                    meta_list.parse_args::<syn::LitInt>().ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("Field `{field_name}` must have a #[tag(N)] attribute"));

        quote! {
            #tag => {
                instance.#field_name = map.next_value()?;
            }
        }
    });

    let expanded = quote! {
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeMap;
                let mut count = 0;
                #(#serialize_count)*
                let mut map = serializer.serialize_map(Some(count))?;
                #(#serialize_fields)*
                map.end()
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("a map with tagged fields")
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        let mut instance = #name::default();
                        while let Some(key) = map.next_key::<u64>()? {
                            match key {
                                #(#deserialize_fields)*
                                _ => {
                                    let _: serde::de::IgnoredAny = map.next_value()?;
                                    continue;
                                }
                            }
                        }
                        Ok(instance)
                    }
                }

                deserializer.deserialize_map(Visitor)
            }
        }
    };

    TokenStream::from(expanded)
}
