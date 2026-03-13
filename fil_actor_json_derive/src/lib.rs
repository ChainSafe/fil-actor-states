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
//! proper JSON objects with named fields:
//!
//! ```text
//! params.to_json_value() → {"to": "f01234", "amount": "1000", "operators": ["f05678"]}
//! ```
//!
//! # Type Recognition
//!
//! The macro recognizes field types by their path suffix and generates appropriate
//! serialization:
//!
//! | Type pattern | JSON output |
//! |---|---|
//! | `Address` | `addr.to_string()` |
//! | `TokenAmount` | `amount.atto().to_string()` |
//! | `BigInt` / `StoragePower` / `DataCap` | `val.to_string()` |
//! | `Cid` | `cid.to_string()` |
//! | `RawBytes` | base64 string |
//! | `PaddedPieceSize` | `.0` (inner u64) |
//! | `Signature` | `{"type": "...", "data": "<base64>"}` |
//! | `ExitCode` | `.value()` |
//! | `Vec<T>` | array of converted elements |
//! | `Option<T>` | null or converted value |
//! | primitives (`u8`..`i64`, `bool`, `String`) | direct JSON value |
//! | other structs | assumes they also impl `to_json_value()` |
//!
//! # Field Attributes
//!
//! - `#[json_value(bigint)]` — treat field as BigInt (calls `.to_string()`)
//! - `#[json_value(serde_fallback)]` — use `serde_json::to_value()` for external types
//!   that can't have IntoJsonValue derived
//!
//! # Struct Attributes
//!
//! - `#[serde(transparent)]` is detected: the generated method delegates to the single field

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, TypePath, parse_macro_input};

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
        let converter = field_to_json_expr(field_name, &field.ty, &field.attrs);
        let expanded = quote! {
            impl #name {
                pub fn to_json_value(&self) -> serde_json::Value {
                    #converter
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
            let converter = field_to_json_expr(field_name, &f.ty, &f.attrs);
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
    };

    expanded.into()
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
                && lit.value().contains("bigint")
            {
                found = true;
            }
            Ok(())
        });
        found
    })
}

/// Generate a token stream expression that converts `self.field_name` to `serde_json::Value`.
fn field_to_json_expr(
    field_name: &syn::Ident,
    ty: &Type,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    // Check for explicit bigint annotation or serde bigint_ser
    if has_json_value_attr(attrs, "bigint") || has_serde_bigint(attrs) {
        return quote! {
            serde_json::Value::String(self.#field_name.to_string())
        };
    }

    // Check for serde_fallback — use serde_json::to_value() for external types
    if has_json_value_attr(attrs, "serde_fallback") {
        return quote! {
            serde_json::to_value(&self.#field_name).unwrap_or(serde_json::Value::Null)
        };
    }

    type_to_json_expr(&quote! { self.#field_name }, ty)
}

/// Classify a type and return the conversion expression.
fn type_to_json_expr(access: &proc_macro2::TokenStream, ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) => path_type_to_json_expr(access, type_path),
        _ => {
            // Fallback: try serde_json::to_value
            quote! { serde_json::to_value(&#access).unwrap_or(serde_json::Value::Null) }
        }
    }
}

fn path_type_to_json_expr(
    access: &proc_macro2::TokenStream,
    type_path: &TypePath,
) -> proc_macro2::TokenStream {
    let last_segment = type_path.path.segments.last().unwrap();
    let type_name = last_segment.ident.to_string();

    match type_name.as_str() {
        // Primitives
        "bool" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "usize"
        | "isize" => {
            quote! { serde_json::json!(#access) }
        }
        "String" => {
            quote! { serde_json::Value::String(#access.clone()) }
        }

        // Filecoin types — Address, BigInt, StoragePower, DataCap, Cid all use .to_string()
        "Address" | "BigInt" | "StoragePower" | "DataCap" | "Cid" => {
            quote! { serde_json::Value::String(#access.to_string()) }
        }
        "TokenAmount" => {
            quote! { serde_json::Value::String(#access.atto().to_string()) }
        }
        "RawBytes" => {
            quote! {{
                use ::base64::Engine as _;
                serde_json::Value::String(
                    ::base64::engine::general_purpose::STANDARD.encode(#access.bytes())
                )
            }}
        }
        "PaddedPieceSize" => {
            quote! { serde_json::json!(#access.0) }
        }
        "Signature" => {
            quote! {{
                use ::base64::Engine as _;
                let mut m = serde_json::Map::new();
                m.insert("type".to_string(),
                    serde_json::Value::String(format!("{:?}", #access.signature_type())));
                m.insert("data".to_string(),
                    serde_json::Value::String(
                        ::base64::engine::general_purpose::STANDARD.encode(#access.bytes())
                    ));
                serde_json::Value::Object(m)
            }}
        }
        "ExitCode" => {
            quote! { serde_json::Value::from(#access.value()) }
        }

        // Generic containers
        "Vec" => {
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
            {
                let inner_convert = type_to_json_expr(&quote! { item }, inner_ty);
                return quote! {
                    serde_json::Value::Array(
                        #access.iter().map(|item| #inner_convert).collect()
                    )
                };
            }
            quote! { serde_json::to_value(&#access).unwrap_or(serde_json::Value::Null) }
        }
        "Option" => {
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
            {
                let inner_convert = type_to_json_expr(&quote! { v }, inner_ty);
                return quote! {
                    match &#access {
                        Some(v) => #inner_convert,
                        None => serde_json::Value::Null,
                    }
                };
            }
            quote! { serde_json::to_value(&#access).unwrap_or(serde_json::Value::Null) }
        }

        // Type aliases we know are numeric
        "ActorID" | "SectorNumber" | "ChainEpoch" | "AllocationID" | "ClaimID" => {
            quote! { serde_json::json!(#access) }
        }

        // Unknown type — assume it has to_json_value()
        _ => {
            quote! { #access.to_json_value() }
        }
    }
}
