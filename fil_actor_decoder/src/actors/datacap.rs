//! DataCap actor (f07) param and return decoder.
//!
//! Supports all actor versions:
//! - v9: legacy method numbers (Mint=2, ..., Allowance=21), only MintParams/DestroyParams types
//! - v10-v11: FRC-0042 method hashes, MintParams/DestroyParams/GranularityReturn
//! - v12-v17: FRC-0042 method hashes, full type set (13 structs, CBOR-identical across versions)

use crate::ActorVersion;
use crate::actors::{cbor_to_json, decode_empty_param};
use anyhow::{Result, bail};
use fvm_ipld_encoding::RawBytes;
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// JSON helpers for frc46_token types (external — no to_json_value())
// ---------------------------------------------------------------------------

fn addr_json(a: &Address) -> Value {
    Value::String(a.to_string())
}

fn token_json(t: &TokenAmount) -> Value {
    Value::String(t.atto().to_string())
}

fn raw_bytes_json(rb: &RawBytes) -> Value {
    use base64::Engine;
    Value::String(base64::engine::general_purpose::STANDARD.encode(rb.bytes()))
}

fn try_decode_nested(payload_type: &str, rb: &RawBytes) -> Value {
    if rb.bytes().is_empty() {
        return json!("");
    }
    if let Ok(decoded) =
        crate::actors::verifreg::decode_nested_payload(payload_type, rb.bytes())
    {
        return decoded;
    }
    raw_bytes_json(rb)
}

// ---------------------------------------------------------------------------
// Method numbers
// ---------------------------------------------------------------------------

/// FRC-0042 method numbers (v10+)
mod methods {
    pub const CONSTRUCTOR: u64 = 1;
    pub const MINT: u64 = frc42_dispatch::method_hash!("Mint");
    pub const DESTROY: u64 = frc42_dispatch::method_hash!("Destroy");
    pub const NAME: u64 = frc42_dispatch::method_hash!("Name");
    pub const SYMBOL: u64 = frc42_dispatch::method_hash!("Symbol");
    pub const GRANULARITY: u64 = frc42_dispatch::method_hash!("Granularity");
    pub const TOTAL_SUPPLY: u64 = frc42_dispatch::method_hash!("TotalSupply");
    pub const BALANCE: u64 = frc42_dispatch::method_hash!("Balance");
    pub const TRANSFER: u64 = frc42_dispatch::method_hash!("Transfer");
    pub const TRANSFER_FROM: u64 = frc42_dispatch::method_hash!("TransferFrom");
    pub const INCREASE_ALLOWANCE: u64 = frc42_dispatch::method_hash!("IncreaseAllowance");
    pub const DECREASE_ALLOWANCE: u64 = frc42_dispatch::method_hash!("DecreaseAllowance");
    pub const REVOKE_ALLOWANCE: u64 = frc42_dispatch::method_hash!("RevokeAllowance");
    pub const BURN: u64 = frc42_dispatch::method_hash!("Burn");
    pub const BURN_FROM: u64 = frc42_dispatch::method_hash!("BurnFrom");
    pub const ALLOWANCE: u64 = frc42_dispatch::method_hash!("Allowance");
}

/// Legacy method numbers (v9 only)
mod v9_methods {
    pub const CONSTRUCTOR: u64 = 0;
    pub const MINT: u64 = 2;
    pub const DESTROY: u64 = 3;
    pub const NAME: u64 = 10;
    pub const SYMBOL: u64 = 11;
    pub const TOTAL_SUPPLY: u64 = 12;
    pub const BALANCE_OF: u64 = 13;
    pub const TRANSFER: u64 = 14;
    pub const TRANSFER_FROM: u64 = 15;
    pub const INCREASE_ALLOWANCE: u64 = 16;
    pub const DECREASE_ALLOWANCE: u64 = 17;
    pub const REVOKE_ALLOWANCE: u64 = 18;
    pub const BURN: u64 = 19;
    pub const BURN_FROM: u64 = 20;
    pub const ALLOWANCE: u64 = 21;
}

// ---------------------------------------------------------------------------
// frc46_token type decoders (external — no derive)
// ---------------------------------------------------------------------------

fn decode_transfer_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::TransferParams;
    let p: TransferParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "to": addr_json(&p.to),
        "amount": token_json(&p.amount),
        "operator_data": try_decode_nested("allocation-requests", &p.operator_data),
    }))
}

fn decode_transfer_from_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::TransferFromParams;
    let p: TransferFromParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "from": addr_json(&p.from),
        "to": addr_json(&p.to),
        "amount": token_json(&p.amount),
        "operator_data": try_decode_nested("allocation-requests", &p.operator_data),
    }))
}

fn decode_increase_allowance_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::IncreaseAllowanceParams;
    let p: IncreaseAllowanceParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "operator": addr_json(&p.operator),
        "increase": token_json(&p.increase),
    }))
}

fn decode_decrease_allowance_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::DecreaseAllowanceParams;
    let p: DecreaseAllowanceParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "operator": addr_json(&p.operator),
        "decrease": token_json(&p.decrease),
    }))
}

fn decode_revoke_allowance_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::RevokeAllowanceParams;
    let p: RevokeAllowanceParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({ "operator": addr_json(&p.operator) }))
}

fn decode_burn_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::BurnParams;
    let p: BurnParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({ "amount": token_json(&p.amount) }))
}

fn decode_burn_from_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::BurnFromParams;
    let p: BurnFromParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "owner": addr_json(&p.owner),
        "amount": token_json(&p.amount),
    }))
}

fn decode_get_allowance_params(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::GetAllowanceParams;
    let p: GetAllowanceParams = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "owner": addr_json(&p.owner),
        "operator": addr_json(&p.operator),
    }))
}

fn decode_transfer_return(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::TransferReturn;
    let r: TransferReturn = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "from_balance": token_json(&r.from_balance),
        "to_balance": token_json(&r.to_balance),
        "recipient_data": try_decode_nested("allocations-response", &r.recipient_data),
    }))
}

fn decode_transfer_from_return(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::TransferFromReturn;
    let r: TransferFromReturn = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "from_balance": token_json(&r.from_balance),
        "to_balance": token_json(&r.to_balance),
        "allowance": token_json(&r.allowance),
        "recipient_data": try_decode_nested("allocations-response", &r.recipient_data),
    }))
}

fn decode_burn_return(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::BurnReturn;
    let r: BurnReturn = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({ "balance": token_json(&r.balance) }))
}

fn decode_burn_from_return(bytes: &[u8]) -> Result<Value> {
    use fil_actors_shared::frc46_token::token::types::BurnFromReturn;
    let r: BurnFromReturn = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({
        "balance": token_json(&r.balance),
        "allowance": token_json(&r.allowance),
    }))
}

fn decode_allowance_return(bytes: &[u8]) -> Result<Value> {
    let r: TokenAmount = fvm_ipld_encoding::from_slice(bytes)?;
    Ok(json!({ "allowance": token_json(&r) }))
}

// ---------------------------------------------------------------------------
// Version-specific dispatch helpers
// ---------------------------------------------------------------------------

/// Decode params for v9 (legacy method numbers, only MintParams/DestroyParams types).
fn decode_params_v9(method_num: u64, bytes: &[u8]) -> Result<Value> {
    use v9_methods::*;
    match method_num {
        MINT => cbor_to_json!(fil_actor_datacap_state::v9::MintParams, bytes),
        DESTROY => cbor_to_json!(fil_actor_datacap_state::v9::DestroyParams, bytes),
        // frc46_token types (shared across versions)
        TRANSFER => decode_transfer_params(bytes),
        TRANSFER_FROM => decode_transfer_from_params(bytes),
        INCREASE_ALLOWANCE => decode_increase_allowance_params(bytes),
        DECREASE_ALLOWANCE => decode_decrease_allowance_params(bytes),
        REVOKE_ALLOWANCE => decode_revoke_allowance_params(bytes),
        BURN => decode_burn_params(bytes),
        BURN_FROM => decode_burn_from_params(bytes),
        ALLOWANCE => decode_get_allowance_params(bytes),
        BALANCE_OF => {
            // v9 BalanceOf takes a raw Address (no wrapper type)
            let addr: fvm_shared4::address::Address = fvm_ipld_encoding::from_slice(bytes)?;
            Ok(json!(addr.to_string()))
        }
        CONSTRUCTOR | NAME | SYMBOL | TOTAL_SUPPLY => decode_empty_param(bytes),
        _ => bail!("Unknown datacap v9 method number: {method_num}"),
    }
}

/// Decode returns for v9 (legacy method numbers).
fn decode_return_v9(method_num: u64, bytes: &[u8]) -> Result<Value> {
    use v9_methods::*;
    match method_num {
        TRANSFER => decode_transfer_return(bytes),
        TRANSFER_FROM => decode_transfer_from_return(bytes),
        BURN => decode_burn_return(bytes),
        BURN_FROM => decode_burn_from_return(bytes),
        INCREASE_ALLOWANCE | DECREASE_ALLOWANCE | REVOKE_ALLOWANCE | ALLOWANCE => {
            decode_allowance_return(bytes)
        }
        // v9 has no typed returns for Name/Symbol/TotalSupply/Balance — raw value
        MINT | DESTROY | CONSTRUCTOR | NAME | SYMBOL | TOTAL_SUPPLY | BALANCE_OF => {
            decode_empty_param(bytes)
        }
        _ => bail!("Return decoding not implemented for datacap v9 method {method_num}"),
    }
}

/// Decode params for v10-v11 (FRC-0042 hashes, limited types).
/// MintParams/DestroyParams available; other typed params don't exist yet.
fn decode_params_v10(method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        // v10 has MintParams, DestroyParams (CBOR-identical to v12+)
        MINT => cbor_to_json!(fil_actor_datacap_state::v10::MintParams, bytes),
        DESTROY => cbor_to_json!(fil_actor_datacap_state::v10::DestroyParams, bytes),
        // frc46_token types (shared across versions)
        TRANSFER => decode_transfer_params(bytes),
        TRANSFER_FROM => decode_transfer_from_params(bytes),
        INCREASE_ALLOWANCE => decode_increase_allowance_params(bytes),
        DECREASE_ALLOWANCE => decode_decrease_allowance_params(bytes),
        REVOKE_ALLOWANCE => decode_revoke_allowance_params(bytes),
        BURN => decode_burn_params(bytes),
        BURN_FROM => decode_burn_from_params(bytes),
        ALLOWANCE => decode_get_allowance_params(bytes),
        // v10-v11 has no ConstructorParams, BalanceParams — treat as raw
        CONSTRUCTOR | BALANCE | NAME | SYMBOL | TOTAL_SUPPLY | GRANULARITY => {
            decode_empty_param(bytes)
        }
        _ => bail!("Unknown datacap v10 method number: {method_num}"),
    }
}

/// Decode returns for v10-v11 (FRC-0042 hashes, limited return types).
fn decode_return_v10(method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        // v10 added GranularityReturn
        GRANULARITY => cbor_to_json!(fil_actor_datacap_state::v10::GranularityReturn, bytes),
        TRANSFER => decode_transfer_return(bytes),
        TRANSFER_FROM => decode_transfer_from_return(bytes),
        BURN => decode_burn_return(bytes),
        BURN_FROM => decode_burn_from_return(bytes),
        INCREASE_ALLOWANCE | DECREASE_ALLOWANCE | REVOKE_ALLOWANCE | ALLOWANCE => {
            decode_allowance_return(bytes)
        }
        // v10-v11 has no NameReturn/SymbolReturn/etc. — treat as raw
        MINT | DESTROY | CONSTRUCTOR | NAME | SYMBOL | TOTAL_SUPPLY | BALANCE => {
            decode_empty_param(bytes)
        }
        _ => bail!("Return decoding not implemented for datacap v10 method {method_num}"),
    }
}

/// Decode params for v12+ (FRC-0042 hashes, full type set).
/// v12-v17 types are CBOR-identical; we use v17 types for all.
fn decode_params_v12plus(method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        CONSTRUCTOR => cbor_to_json!(fil_actor_datacap_state::v17::ConstructorParams, bytes),
        BALANCE => cbor_to_json!(fil_actor_datacap_state::v17::BalanceParams, bytes),
        MINT => cbor_to_json!(fil_actor_datacap_state::v17::MintParams, bytes),
        DESTROY => cbor_to_json!(fil_actor_datacap_state::v17::DestroyParams, bytes),
        TRANSFER => decode_transfer_params(bytes),
        TRANSFER_FROM => decode_transfer_from_params(bytes),
        INCREASE_ALLOWANCE => decode_increase_allowance_params(bytes),
        DECREASE_ALLOWANCE => decode_decrease_allowance_params(bytes),
        REVOKE_ALLOWANCE => decode_revoke_allowance_params(bytes),
        BURN => decode_burn_params(bytes),
        BURN_FROM => decode_burn_from_params(bytes),
        ALLOWANCE => decode_get_allowance_params(bytes),
        NAME | SYMBOL | TOTAL_SUPPLY | GRANULARITY => decode_empty_param(bytes),
        _ => bail!("Unknown datacap method number: {method_num}"),
    }
}

/// Decode returns for v12+ (FRC-0042 hashes, full return types).
fn decode_return_v12plus(method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        NAME => cbor_to_json!(fil_actor_datacap_state::v17::NameReturn, bytes),
        SYMBOL => cbor_to_json!(fil_actor_datacap_state::v17::SymbolReturn, bytes),
        TOTAL_SUPPLY => cbor_to_json!(fil_actor_datacap_state::v17::TotalSupplyReturn, bytes),
        BALANCE => cbor_to_json!(fil_actor_datacap_state::v17::BalanceReturn, bytes),
        GRANULARITY => cbor_to_json!(fil_actor_datacap_state::v17::GranularityReturn, bytes),
        TRANSFER => decode_transfer_return(bytes),
        TRANSFER_FROM => decode_transfer_from_return(bytes),
        BURN => decode_burn_return(bytes),
        BURN_FROM => decode_burn_from_return(bytes),
        MINT | DESTROY | CONSTRUCTOR => decode_empty_param(bytes),
        INCREASE_ALLOWANCE | DECREASE_ALLOWANCE | REVOKE_ALLOWANCE | ALLOWANCE => {
            decode_allowance_return(bytes)
        }
        _ => bail!("Return decoding not implemented for datacap method {method_num}"),
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn decode_params(version: ActorVersion, method_num: u64, bytes: &[u8]) -> Result<Value> {
    match version {
        ActorVersion::V9 => decode_params_v9(method_num, bytes),
        ActorVersion::V10 | ActorVersion::V11 => decode_params_v10(method_num, bytes),
        ActorVersion::V12
        | ActorVersion::V13
        | ActorVersion::V14
        | ActorVersion::V15
        | ActorVersion::V16
        | ActorVersion::V17 => decode_params_v12plus(method_num, bytes),
    }
}

pub fn decode_return(version: ActorVersion, method_num: u64, bytes: &[u8]) -> Result<Value> {
    match version {
        ActorVersion::V9 => decode_return_v9(method_num, bytes),
        ActorVersion::V10 | ActorVersion::V11 => decode_return_v10(method_num, bytes),
        ActorVersion::V12
        | ActorVersion::V13
        | ActorVersion::V14
        | ActorVersion::V15
        | ActorVersion::V16
        | ActorVersion::V17 => decode_return_v12plus(method_num, bytes),
    }
}

// ---------------------------------------------------------------------------
// Method name lookup
// ---------------------------------------------------------------------------

pub fn method_name(method_num: u64) -> &'static str {
    match method_num {
        // Legacy v9 method numbers
        v9_methods::CONSTRUCTOR => "Constructor",
        v9_methods::MINT => "Mint",
        v9_methods::DESTROY => "Destroy",
        v9_methods::NAME => "Name",
        v9_methods::SYMBOL => "Symbol",
        v9_methods::TOTAL_SUPPLY => "TotalSupply",
        v9_methods::BALANCE_OF => "BalanceOf",
        v9_methods::TRANSFER => "Transfer",
        v9_methods::TRANSFER_FROM => "TransferFrom",
        v9_methods::INCREASE_ALLOWANCE => "IncreaseAllowance",
        v9_methods::DECREASE_ALLOWANCE => "DecreaseAllowance",
        v9_methods::REVOKE_ALLOWANCE => "RevokeAllowance",
        v9_methods::BURN => "Burn",
        v9_methods::BURN_FROM => "BurnFrom",
        v9_methods::ALLOWANCE => "Allowance",
        // v10+ constructor
        methods::CONSTRUCTOR => "Constructor",
        // FRC-0042 method hashes
        m if m == methods::MINT => "Mint",
        m if m == methods::DESTROY => "Destroy",
        m if m == methods::NAME => "Name",
        m if m == methods::SYMBOL => "Symbol",
        m if m == methods::GRANULARITY => "Granularity",
        m if m == methods::TOTAL_SUPPLY => "TotalSupply",
        m if m == methods::BALANCE => "Balance",
        m if m == methods::TRANSFER => "Transfer",
        m if m == methods::TRANSFER_FROM => "TransferFrom",
        m if m == methods::INCREASE_ALLOWANCE => "IncreaseAllowance",
        m if m == methods::DECREASE_ALLOWANCE => "DecreaseAllowance",
        m if m == methods::REVOKE_ALLOWANCE => "RevokeAllowance",
        m if m == methods::BURN => "Burn",
        m if m == methods::BURN_FROM => "BurnFrom",
        m if m == methods::ALLOWANCE => "Allowance",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fvm_ipld_encoding::to_vec;

    #[test]
    fn test_decode_mint_params_v17() {
        let params = fil_actor_datacap_state::v17::MintParams {
            to: Address::new_id(1234),
            amount: TokenAmount::from_atto(1_000_000_000_000_000_000_i64),
            operators: vec![Address::new_id(5678)],
        };
        let cbor = to_vec(&params).unwrap();
        let result = decode_params(ActorVersion::V17, methods::MINT, &cbor).unwrap();
        assert_eq!(result["to"], "f01234");
        assert_eq!(result["amount"], "1000000000000000000");
        assert_eq!(result["operators"][0], "f05678");
    }

    #[test]
    fn test_decode_mint_params_v9() {
        // v9 uses fvm_shared v2 Address — construct via cbor roundtrip from v17
        let params_v17 = fil_actor_datacap_state::v17::MintParams {
            to: Address::new_id(1234),
            amount: TokenAmount::from_atto(1_000_000_000_000_000_000_i64),
            operators: vec![Address::new_id(5678)],
        };
        // CBOR encoding is identical between v9 and v17 MintParams
        let cbor = to_vec(&params_v17).unwrap();
        let result = decode_params(ActorVersion::V9, v9_methods::MINT, &cbor).unwrap();
        assert_eq!(result["to"], "f01234");
    }

    #[test]
    fn test_decode_destroy_params() {
        let params = fil_actor_datacap_state::v17::DestroyParams {
            owner: Address::new_id(100),
            amount: TokenAmount::from_atto(500),
        };
        let cbor = to_vec(&params).unwrap();
        let result = decode_params(ActorVersion::V17, methods::DESTROY, &cbor).unwrap();
        assert_eq!(result["owner"], "f0100");
        assert_eq!(result["amount"], "500");
    }

    #[test]
    fn test_decode_empty_params() {
        let result = decode_params(ActorVersion::V17, methods::NAME, &[]).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_decode_balance_params() {
        let params = fil_actor_datacap_state::v17::BalanceParams {
            address: Address::new_id(42),
        };
        let cbor = to_vec(&params).unwrap();
        let result = decode_params(ActorVersion::V17, methods::BALANCE, &cbor).unwrap();
        assert_eq!(result, json!("f042"));
    }

    #[test]
    fn test_v12_uses_v12plus_dispatch() {
        // v12 should use the same dispatch as v17
        let result = decode_params(ActorVersion::V12, methods::NAME, &[]).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_unknown_method_error() {
        assert!(decode_params(ActorVersion::V17, 99999, &[]).is_err());
    }

    #[test]
    fn test_method_name() {
        assert_eq!(method_name(methods::TRANSFER), "Transfer");
        assert_eq!(method_name(methods::MINT), "Mint");
        assert_eq!(method_name(v9_methods::MINT), "Mint");
        assert_eq!(method_name(v9_methods::BALANCE_OF), "BalanceOf");
        assert_eq!(method_name(99999), "Unknown");
    }
}
