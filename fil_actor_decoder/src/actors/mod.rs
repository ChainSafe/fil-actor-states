pub mod datacap;
pub mod verifreg;

use anyhow::Result;
use serde_json::{Value, json};

macro_rules! cbor_to_json {
    ($ty:ty, $bytes:expr) => {{
        let val: $ty = fvm_ipld_encoding::from_slice($bytes)?;
        Ok(val.to_json_value())
    }};
}
pub(crate) use cbor_to_json;

pub(crate) fn decode_empty_param(bytes: &[u8]) -> Result<Value> {
    if bytes.is_empty() {
        Ok(json!({}))
    } else {
        use base64::Engine;
        Ok(json!({ "raw": base64::engine::general_purpose::STANDARD.encode(bytes) }))
    }
}
