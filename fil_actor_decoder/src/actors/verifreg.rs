//! Verified Registry actor (f06) param and return decoder.
//!
//! Supports all actor versions:
//! - v9: numeric + FRC-0042 methods, flat SectorAllocationClaim, Address-based provider
//! - v10-v11: same methods, flat SectorAllocationClaim, ActorID-based provider
//! - v12-v17: same methods, nested SectorAllocationClaims (CBOR-identical across v12-v17)

use crate::ActorVersion;
use crate::actors::{cbor_to_json, decode_empty_param};
use anyhow::{Result, bail};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Method numbers (shared across v9-v17)
// ---------------------------------------------------------------------------

mod methods {
    pub const CONSTRUCTOR: u64 = 0;
    pub const ADD_VERIFIER: u64 = 2;
    pub const REMOVE_VERIFIER: u64 = 3;
    pub const ADD_VERIFIED_CLIENT: u64 = 4;
    pub const REMOVE_VERIFIED_CLIENT_DATA_CAP: u64 = 7;
    pub const REMOVE_EXPIRED_ALLOCATIONS: u64 = 8;
    pub const CLAIM_ALLOCATIONS: u64 = 9;
    pub const GET_CLAIMS: u64 = 10;
    pub const EXTEND_CLAIM_TERMS: u64 = 11;
    pub const REMOVE_EXPIRED_CLAIMS: u64 = 12;

    // FRC-0042 exported methods
    pub const ADD_VERIFIED_CLIENT_EXPORTED: u64 =
        frc42_dispatch::method_hash!("AddVerifiedClient");
    pub const REMOVE_EXPIRED_ALLOCATIONS_EXPORTED: u64 =
        frc42_dispatch::method_hash!("RemoveExpiredAllocations");
    pub const GET_CLAIMS_EXPORTED: u64 = frc42_dispatch::method_hash!("GetClaims");
    pub const EXTEND_CLAIM_TERMS_EXPORTED: u64 =
        frc42_dispatch::method_hash!("ExtendClaimTerms");
    pub const REMOVE_EXPIRED_CLAIMS_EXPORTED: u64 =
        frc42_dispatch::method_hash!("RemoveExpiredClaims");
    pub const UNIVERSAL_RECEIVER_HOOK: u64 = frc42_dispatch::method_hash!("Receive");
}

/// Decode nested payloads found inside operator_data / recipient_data.
/// Uses v17 types (CBOR-identical for v12+; v9-v11 have different AllocationRequests
/// but we attempt v17 first and fall back gracefully in the caller).
pub fn decode_nested_payload(payload_type: &str, bytes: &[u8]) -> Result<Value> {
    match payload_type {
        "allocation-requests" => {
            cbor_to_json!(fil_actor_verifreg_state::v17::AllocationRequests, bytes)
        }
        "allocations-response" => {
            cbor_to_json!(fil_actor_verifreg_state::v17::AllocationsResponse, bytes)
        }
        _ => bail!("Unknown nested payload type: {payload_type}"),
    }
}

// ---------------------------------------------------------------------------
// Version-specific dispatch: params
// ---------------------------------------------------------------------------

/// Shared params that are identical across v9-v17 (same CBOR layout).
fn decode_shared_params(method_num: u64, bytes: &[u8]) -> Result<Option<Value>> {
    use methods::*;
    let val = match method_num {
        ADD_VERIFIER | ADD_VERIFIED_CLIENT | ADD_VERIFIED_CLIENT_EXPORTED => {
            let p: fil_actor_verifreg_state::v17::VerifierParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        REMOVE_VERIFIER => {
            let p: fil_actor_verifreg_state::v17::RemoveVerifierParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        REMOVE_VERIFIED_CLIENT_DATA_CAP => {
            let p: fil_actor_verifreg_state::v17::RemoveDataCapParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        REMOVE_EXPIRED_ALLOCATIONS | REMOVE_EXPIRED_ALLOCATIONS_EXPORTED => {
            let p: fil_actor_verifreg_state::v17::RemoveExpiredAllocationsParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        GET_CLAIMS | GET_CLAIMS_EXPORTED => {
            let p: fil_actor_verifreg_state::v17::GetClaimsParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        EXTEND_CLAIM_TERMS | EXTEND_CLAIM_TERMS_EXPORTED => {
            let p: fil_actor_verifreg_state::v17::ExtendClaimTermsParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        REMOVE_EXPIRED_CLAIMS | REMOVE_EXPIRED_CLAIMS_EXPORTED => {
            let p: fil_actor_verifreg_state::v17::RemoveExpiredClaimsParams =
                fvm_ipld_encoding::from_slice(bytes)?;
            p.to_json_value()
        }
        CONSTRUCTOR => return Ok(Some(decode_empty_param(bytes)?)),
        _ => return Ok(None),
    };
    Ok(Some(val))
}

fn decode_params_v9(method_num: u64, bytes: &[u8]) -> Result<Value> {
    if let Some(v) = decode_shared_params(method_num, bytes)? {
        return Ok(v);
    }
    use methods::*;
    match method_num {
        // v9 has flat SectorAllocationClaim and Address-based AllocationRequest
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v9::ClaimAllocationsParams, bytes)
        }
        UNIVERSAL_RECEIVER_HOOK => {
            cbor_to_json!(fil_actor_verifreg_state::v9::AllocationRequests, bytes)
        }
        _ => bail!("Unknown verifreg v9 method number: {method_num}"),
    }
}

fn decode_params_v10(method_num: u64, bytes: &[u8]) -> Result<Value> {
    if let Some(v) = decode_shared_params(method_num, bytes)? {
        return Ok(v);
    }
    use methods::*;
    match method_num {
        // v10-v11 has flat SectorAllocationClaim but ActorID-based
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v10::ClaimAllocationsParams, bytes)
        }
        UNIVERSAL_RECEIVER_HOOK => {
            cbor_to_json!(fil_actor_verifreg_state::v10::AllocationRequests, bytes)
        }
        _ => bail!("Unknown verifreg v10 method number: {method_num}"),
    }
}

fn decode_params_v12plus(method_num: u64, bytes: &[u8]) -> Result<Value> {
    if let Some(v) = decode_shared_params(method_num, bytes)? {
        return Ok(v);
    }
    use methods::*;
    match method_num {
        // v12+ has nested SectorAllocationClaims
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v17::ClaimAllocationsParams, bytes)
        }
        UNIVERSAL_RECEIVER_HOOK => {
            cbor_to_json!(fil_actor_verifreg_state::v17::AllocationRequests, bytes)
        }
        _ => bail!("Unknown verifreg method number: {method_num}"),
    }
}

// ---------------------------------------------------------------------------
// Version-specific dispatch: returns
// ---------------------------------------------------------------------------

/// Shared returns that are identical across v9-v17.
fn decode_shared_returns(method_num: u64, bytes: &[u8]) -> Result<Option<Value>> {
    use methods::*;
    let val = match method_num {
        REMOVE_VERIFIED_CLIENT_DATA_CAP => {
            let r: fil_actor_verifreg_state::v17::RemoveDataCapReturn =
                fvm_ipld_encoding::from_slice(bytes)?;
            r.to_json_value()
        }
        REMOVE_EXPIRED_ALLOCATIONS | REMOVE_EXPIRED_ALLOCATIONS_EXPORTED => {
            let r: fil_actor_verifreg_state::v17::RemoveExpiredAllocationsReturn =
                fvm_ipld_encoding::from_slice(bytes)?;
            r.to_json_value()
        }
        GET_CLAIMS | GET_CLAIMS_EXPORTED => {
            let r: fil_actor_verifreg_state::v17::GetClaimsReturn =
                fvm_ipld_encoding::from_slice(bytes)?;
            r.to_json_value()
        }
        REMOVE_EXPIRED_CLAIMS | REMOVE_EXPIRED_CLAIMS_EXPORTED => {
            let r: fil_actor_verifreg_state::v17::RemoveExpiredClaimsReturn =
                fvm_ipld_encoding::from_slice(bytes)?;
            r.to_json_value()
        }
        UNIVERSAL_RECEIVER_HOOK => {
            let r: fil_actor_verifreg_state::v17::AllocationsResponse =
                fvm_ipld_encoding::from_slice(bytes)?;
            r.to_json_value()
        }
        CONSTRUCTOR | ADD_VERIFIER | REMOVE_VERIFIER | ADD_VERIFIED_CLIENT
        | ADD_VERIFIED_CLIENT_EXPORTED | EXTEND_CLAIM_TERMS | EXTEND_CLAIM_TERMS_EXPORTED => {
            return Ok(Some(decode_empty_param(bytes)?));
        }
        _ => return Ok(None),
    };
    Ok(Some(val))
}

fn decode_return_v9(method_num: u64, bytes: &[u8]) -> Result<Value> {
    if let Some(v) = decode_shared_returns(method_num, bytes)? {
        return Ok(v);
    }
    use methods::*;
    match method_num {
        // v9 ClaimAllocationsReturn has different structure: {batch_info, claimed_space}
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v9::ClaimAllocationsReturn, bytes)
        }
        _ => bail!("Return decoding not implemented for verifreg v9 method {method_num}"),
    }
}

fn decode_return_v10(method_num: u64, bytes: &[u8]) -> Result<Value> {
    if let Some(v) = decode_shared_returns(method_num, bytes)? {
        return Ok(v);
    }
    use methods::*;
    match method_num {
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v10::ClaimAllocationsReturn, bytes)
        }
        _ => bail!("Return decoding not implemented for verifreg v10 method {method_num}"),
    }
}

fn decode_return_v12plus(method_num: u64, bytes: &[u8]) -> Result<Value> {
    if let Some(v) = decode_shared_returns(method_num, bytes)? {
        return Ok(v);
    }
    use methods::*;
    match method_num {
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v17::ClaimAllocationsReturn, bytes)
        }
        _ => bail!("Return decoding not implemented for verifreg method {method_num}"),
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
        methods::CONSTRUCTOR => "Constructor",
        methods::ADD_VERIFIER => "AddVerifier",
        methods::REMOVE_VERIFIER => "RemoveVerifier",
        methods::ADD_VERIFIED_CLIENT => "AddVerifiedClient",
        methods::REMOVE_VERIFIED_CLIENT_DATA_CAP => "RemoveVerifiedClientDataCap",
        methods::REMOVE_EXPIRED_ALLOCATIONS => "RemoveExpiredAllocations",
        methods::CLAIM_ALLOCATIONS => "ClaimAllocations",
        methods::GET_CLAIMS => "GetClaims",
        methods::EXTEND_CLAIM_TERMS => "ExtendClaimTerms",
        methods::REMOVE_EXPIRED_CLAIMS => "RemoveExpiredClaims",
        m if m == methods::ADD_VERIFIED_CLIENT_EXPORTED => "AddVerifiedClientExported",
        m if m == methods::REMOVE_EXPIRED_ALLOCATIONS_EXPORTED => {
            "RemoveExpiredAllocationsExported"
        }
        m if m == methods::GET_CLAIMS_EXPORTED => "GetClaimsExported",
        m if m == methods::EXTEND_CLAIM_TERMS_EXPORTED => "ExtendClaimTermsExported",
        m if m == methods::REMOVE_EXPIRED_CLAIMS_EXPORTED => "RemoveExpiredClaimsExported",
        m if m == methods::UNIVERSAL_RECEIVER_HOOK => "UniversalReceiverHook",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fvm_ipld_encoding::to_vec;
    use fvm_shared4::address::Address;
    use num_bigint::BigInt;
    use serde_json::json;

    #[test]
    fn test_decode_verifier_params() {
        let params = fil_actor_verifreg_state::v17::VerifierParams {
            address: Address::new_id(1000),
            allowance: BigInt::from(5_000_000),
        };
        let cbor = to_vec(&params).unwrap();
        let result = decode_params(ActorVersion::V17, methods::ADD_VERIFIER, &cbor).unwrap();
        assert_eq!(result["address"], "f01000");
        assert_eq!(result["allowance"], "5000000");
    }

    #[test]
    fn test_decode_get_claims_params() {
        let params = fil_actor_verifreg_state::v17::GetClaimsParams {
            provider: 1234,
            claim_ids: vec![1, 2, 3],
        };
        let cbor = to_vec(&params).unwrap();
        let result = decode_params(ActorVersion::V17, methods::GET_CLAIMS, &cbor).unwrap();
        assert_eq!(result["provider"], 1234);
        assert_eq!(result["claim_ids"], json!([1, 2, 3]));
    }

    #[test]
    fn test_v12_dispatch() {
        let result = decode_params(ActorVersion::V12, methods::CONSTRUCTOR, &[]).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_v9_dispatch() {
        let result = decode_params(ActorVersion::V9, methods::CONSTRUCTOR, &[]).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_decode_empty_constructor() {
        let result = decode_params(ActorVersion::V17, methods::CONSTRUCTOR, &[]).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_unknown_method_error() {
        assert!(decode_params(ActorVersion::V17, 99999, &[]).is_err());
    }

    #[test]
    fn test_method_name() {
        assert_eq!(method_name(methods::ADD_VERIFIER), "AddVerifier");
        assert_eq!(method_name(methods::CLAIM_ALLOCATIONS), "ClaimAllocations");
        assert_eq!(method_name(99999), "Unknown");
    }

    #[test]
    fn test_nested_allocation_requests() {
        let requests = fil_actor_verifreg_state::v17::AllocationRequests {
            allocations: vec![fil_actor_verifreg_state::v17::AllocationRequest {
                provider: 100,
                data: cid::Cid::default(),
                size: fvm_shared4::piece::PaddedPieceSize(1024),
                term_min: 518400,
                term_max: 1036800,
                expiration: 2000000,
            }],
            extensions: vec![],
        };
        let cbor = to_vec(&requests).unwrap();
        let result = decode_nested_payload("allocation-requests", &cbor).unwrap();
        assert_eq!(result["allocations"][0]["provider"], 100);
        assert_eq!(result["allocations"][0]["size"], 1024);
    }
}
