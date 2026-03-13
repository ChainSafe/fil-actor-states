//! DataCap actor (f07) param and return decoder.
//!
//! Supports actor versions v16 and v17.

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
// FRC-0042 method numbers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Dispatch (v16/v17 only)
// ---------------------------------------------------------------------------

pub fn decode_params(_version: ActorVersion, method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        // Types with to_json_value() (from fil_actor_datacap_state)
        CONSTRUCTOR => cbor_to_json!(fil_actor_datacap_state::v17::ConstructorParams, bytes),
        BALANCE => cbor_to_json!(fil_actor_datacap_state::v17::BalanceParams, bytes),
        MINT => cbor_to_json!(fil_actor_datacap_state::v17::MintParams, bytes),
        DESTROY => cbor_to_json!(fil_actor_datacap_state::v17::DestroyParams, bytes),
        // frc46_token types (manual converters)
        TRANSFER => decode_transfer_params(bytes),
        TRANSFER_FROM => decode_transfer_from_params(bytes),
        INCREASE_ALLOWANCE => decode_increase_allowance_params(bytes),
        DECREASE_ALLOWANCE => decode_decrease_allowance_params(bytes),
        REVOKE_ALLOWANCE => decode_revoke_allowance_params(bytes),
        BURN => decode_burn_params(bytes),
        BURN_FROM => decode_burn_from_params(bytes),
        ALLOWANCE => decode_get_allowance_params(bytes),
        // No params
        NAME | SYMBOL | TOTAL_SUPPLY | GRANULARITY => decode_empty_param(bytes),
        _ => bail!("Unknown datacap method number: {method_num}"),
    }
}

pub fn decode_return(_version: ActorVersion, method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        // Types with to_json_value()
        NAME => cbor_to_json!(fil_actor_datacap_state::v17::NameReturn, bytes),
        SYMBOL => cbor_to_json!(fil_actor_datacap_state::v17::SymbolReturn, bytes),
        TOTAL_SUPPLY => cbor_to_json!(fil_actor_datacap_state::v17::TotalSupplyReturn, bytes),
        BALANCE => cbor_to_json!(fil_actor_datacap_state::v17::BalanceReturn, bytes),
        GRANULARITY => cbor_to_json!(fil_actor_datacap_state::v17::GranularityReturn, bytes),
        // frc46_token returns
        TRANSFER => decode_transfer_return(bytes),
        TRANSFER_FROM => decode_transfer_from_return(bytes),
        BURN => decode_burn_return(bytes),
        BURN_FROM => decode_burn_from_return(bytes),
        // No meaningful return
        MINT | DESTROY | CONSTRUCTOR => decode_empty_param(bytes),
        INCREASE_ALLOWANCE | DECREASE_ALLOWANCE | REVOKE_ALLOWANCE | ALLOWANCE => {
            let r: TokenAmount = fvm_ipld_encoding::from_slice(bytes)?;
            Ok(json!({ "allowance": token_json(&r) }))
        }
        _ => bail!("Return decoding not implemented for datacap method {method_num}"),
    }
}

// ---------------------------------------------------------------------------
// Method name lookup
// ---------------------------------------------------------------------------

pub fn method_name(method_num: u64) -> &'static str {
    match method_num {
        methods::CONSTRUCTOR => "Constructor",
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
    fn test_decode_mint_params() {
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
        // BalanceParams is #[serde(transparent)] — serializes as just the address
        let params = fil_actor_datacap_state::v17::BalanceParams {
            address: Address::new_id(42),
        };
        let cbor = to_vec(&params).unwrap();
        let result = decode_params(ActorVersion::V17, methods::BALANCE, &cbor).unwrap();
        assert_eq!(result, json!("f042"));
    }

    #[test]
    fn test_unknown_method_error() {
        assert!(decode_params(ActorVersion::V17, 99999, &[]).is_err());
    }

    #[test]
    fn test_method_name() {
        assert_eq!(method_name(methods::TRANSFER), "Transfer");
        assert_eq!(method_name(methods::MINT), "Mint");
        assert_eq!(method_name(99999), "Unknown");
    }
}
