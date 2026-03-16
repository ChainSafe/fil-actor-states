//! Derive macro for converting `Serialize_tuple` Filecoin actor types into
//! `serde_json::Value` objects with named fields.
//!
//! # Problem
//!
//! Filecoin actor types use `#[derive(Serialize_tuple)]` which serializes structs
//! as CBOR arrays (positional, no field names). When you call `serde_json::to_value()`
//! on these types, you get JSON arrays instead of objects:
//!
//! ```text
//! // MintParams { to: f01234, amount: 1000, operators: [f05678] }
//! serde_json::to_value() → ["f01234", "1000", ["f05678"]]   // no field names!
//! ```
//!
//! # Solution
//!
//! `#[derive(IntoJsonValue)]` generates a `to_json_value()` method that produces
//! proper JSON objects with named fields. Each field is converted via the
//! `JsonField` trait (from `fil_actors_shared::json_field`).
//!
//! ```text
//! params.to_json_value() → {"to": "f01234", "amount": "1000", "operators": ["f05678"]}
//! ```
//!
//! # Adding New Types
//!
//! To support a new field type, implement `JsonField` for it in
//! `fil_actors_shared/src/json_field.rs`. No macro changes needed.
//!
//! # Field Attributes
//!
//! - `#[json_value(bigint)]` — treat field as BigInt (calls `.to_string()`)
//! - Auto-detected: `#[serde(with = "bigint_ser")]` has the same effect
//!
//! # Struct Attributes
//!
//! - `#[serde(transparent)]` is detected: the generated method delegates to the single field

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(IntoJsonValue, attributes(json_value))]
pub fn derive_into_json_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return syn::Error::new_spanned(&input, "IntoJsonValue only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let fields = match &data.fields {
        Fields::Named(f) => &f.named,
        _ => {
            return syn::Error::new_spanned(
                &input,
                "IntoJsonValue only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    // Check for #[serde(transparent)]
    let is_transparent = input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("serde") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("transparent") {
                found = true;
            }
            Ok(())
        });
        found
    });

    if is_transparent && fields.len() == 1 {
        let field = fields.first().unwrap();
        let field_name = field.ident.as_ref().unwrap();
        let converter = field_converter(field_name, &field.attrs);
        let expanded = quote! {
            impl #name {
                pub fn to_json_value(&self) -> serde_json::Value {
                    #converter
                }
            }
            impl fil_actors_shared::json_field::JsonField for #name {
                fn to_json_field(&self) -> serde_json::Value {
                    self.to_json_value()
                }
            }
        };
        return expanded.into();
    }

    // Regular struct: produce a JSON object with named fields
    let field_count = fields.len();
    let field_entries: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let converter = field_converter(field_name, &f.attrs);
            quote! {
                map.insert(#field_name_str.to_string(), #converter);
            }
        })
        .collect();

    let expanded = quote! {
        impl #name {
            pub fn to_json_value(&self) -> serde_json::Value {
                let mut map = serde_json::Map::with_capacity(#field_count);
                #(#field_entries)*
                serde_json::Value::Object(map)
            }
        }
        impl fil_actors_shared::json_field::JsonField for #name {
            fn to_json_field(&self) -> serde_json::Value {
                self.to_json_value()
            }
        }
    };

    expanded.into()
}

/// Generate the conversion expression for a single field.
///
/// - BigInt-annotated fields use `BigIntJsonField::bigint_to_json_field()`
/// - All other fields use `JsonField::to_json_field()`
fn field_converter(
    field_name: &syn::Ident,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    if has_json_value_attr(attrs, "bigint") || has_serde_bigint(attrs) {
        quote! {
            fil_actors_shared::json_field::BigIntJsonField::bigint_to_json_field(&self.#field_name)
        }
    } else {
        quote! {
            fil_actors_shared::json_field::JsonField::to_json_field(&self.#field_name)
        }
    }
}

/// Check if a field has a specific `#[json_value(...)]` attribute
fn has_json_value_attr(attrs: &[syn::Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("json_value") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(attr_name) {
                found = true;
            }
            Ok(())
        });
        found
    })
}

/// Check if a field has `#[serde(with = "bigint_ser")]`
fn has_serde_bigint(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("serde") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("with")
                && let Ok(lit) = meta.value().and_then(|v| v.parse::<syn::LitStr>())
                && lit.value().ends_with("bigint_ser")
            {
                found = true;
            }
            Ok(())
        });
        found
    })
}
