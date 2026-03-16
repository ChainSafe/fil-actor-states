//! Trait-based JSON field conversion for Filecoin actor types.
//!
//! Provides [`JsonField`] — a trait that converts values to `serde_json::Value`
//! with Filecoin-appropriate representations (e.g., `Address` → string,
//! `TokenAmount` → atto string, `Cid` → CID string).
//!
//! Adding a new Filecoin type only requires a trait impl here — no derive macro
//! changes needed.

use serde_json::Value;

/// Convert a value to a JSON field representation.
///
/// Implement this trait for any type that appears as a struct field in
/// `#[derive(IntoJsonValue)]` types. The derive macro calls `.to_json_field()`
/// on each field.
pub trait JsonField {
    fn to_json_field(&self) -> Value;
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

impl JsonField for bool {
    fn to_json_field(&self) -> Value {
        Value::Bool(*self)
    }
}

impl JsonField for String {
    fn to_json_field(&self) -> Value {
        Value::String(self.clone())
    }
}

macro_rules! impl_json_field_number {
    ($($t:ty),*) => {
        $(
            impl JsonField for $t {
                fn to_json_field(&self) -> Value {
                    serde_json::json!(*self)
                }
            }
        )*
    };
}

impl_json_field_number!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

impl<T: JsonField> JsonField for Vec<T> {
    fn to_json_field(&self) -> Value {
        Value::Array(self.iter().map(|item| item.to_json_field()).collect())
    }
}

impl<T: JsonField> JsonField for Option<T> {
    fn to_json_field(&self) -> Value {
        match self {
            Some(v) => v.to_json_field(),
            None => Value::Null,
        }
    }
}

// ---------------------------------------------------------------------------
// Filecoin / FVM types (from fvm_shared4)
// ---------------------------------------------------------------------------

impl JsonField for fvm_shared4::address::Address {
    fn to_json_field(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl JsonField for fvm_shared4::econ::TokenAmount {
    fn to_json_field(&self) -> Value {
        Value::String(self.atto().to_string())
    }
}

impl JsonField for fvm_shared4::piece::PaddedPieceSize {
    fn to_json_field(&self) -> Value {
        serde_json::json!(self.0)
    }
}

impl JsonField for fvm_shared4::crypto::signature::Signature {
    fn to_json_field(&self) -> Value {
        use base64::Engine;
        let mut m = serde_json::Map::with_capacity(2);
        m.insert(
            "type".to_string(),
            Value::String(format!("{:?}", self.signature_type())),
        );
        m.insert(
            "data".to_string(),
            Value::String(base64::engine::general_purpose::STANDARD.encode(self.bytes())),
        );
        Value::Object(m)
    }
}

impl JsonField for fvm_shared4::error::ExitCode {
    fn to_json_field(&self) -> Value {
        Value::from(self.value())
    }
}

// BigInt (num_bigint)
impl JsonField for num_bigint::BigInt {
    fn to_json_field(&self) -> Value {
        Value::String(self.to_string())
    }
}

// Cid
impl JsonField for cid::Cid {
    fn to_json_field(&self) -> Value {
        Value::String(self.to_string())
    }
}

// RawBytes
impl JsonField for fvm_ipld_encoding::RawBytes {
    fn to_json_field(&self) -> Value {
        use base64::Engine;
        Value::String(base64::engine::general_purpose::STANDARD.encode(self.bytes()))
    }
}

// ---------------------------------------------------------------------------
// Marker trait for BigInt-like types serialized with `#[serde(with = "bigint_ser")]`.
// The derive macro uses `BigIntJsonField::bigint_to_json_field()` for these.
// ---------------------------------------------------------------------------

/// Trait used by the derive macro for fields annotated with `#[json_value(bigint)]`
/// or `#[serde(with = "bigint_ser")]`.
pub trait BigIntJsonField {
    fn bigint_to_json_field(&self) -> Value;
}

impl<T: std::fmt::Display> BigIntJsonField for T {
    fn bigint_to_json_field(&self) -> Value {
        Value::String(self.to_string())
    }
}
