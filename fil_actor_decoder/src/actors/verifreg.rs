//! Verified Registry actor (f06) param and return decoder.
//!
//! Supports actor versions v16 and v17.

use crate::ActorVersion;
use crate::actors::{cbor_to_json, decode_empty_param};
use anyhow::{Result, bail};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Method numbers (v16/v17 share the same set)
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
// Dispatch (v16/v17)
// ---------------------------------------------------------------------------

pub fn decode_params(_version: ActorVersion, method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        ADD_VERIFIER | ADD_VERIFIED_CLIENT | ADD_VERIFIED_CLIENT_EXPORTED => {
            cbor_to_json!(fil_actor_verifreg_state::v17::VerifierParams, bytes)
        }
        REMOVE_VERIFIER => {
            cbor_to_json!(fil_actor_verifreg_state::v17::RemoveVerifierParams, bytes)
        }
        REMOVE_VERIFIED_CLIENT_DATA_CAP => {
            cbor_to_json!(fil_actor_verifreg_state::v17::RemoveDataCapParams, bytes)
        }
        REMOVE_EXPIRED_ALLOCATIONS | REMOVE_EXPIRED_ALLOCATIONS_EXPORTED => {
            cbor_to_json!(
                fil_actor_verifreg_state::v17::RemoveExpiredAllocationsParams,
                bytes
            )
        }
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v17::ClaimAllocationsParams, bytes)
        }
        GET_CLAIMS | GET_CLAIMS_EXPORTED => {
            cbor_to_json!(fil_actor_verifreg_state::v17::GetClaimsParams, bytes)
        }
        EXTEND_CLAIM_TERMS | EXTEND_CLAIM_TERMS_EXPORTED => {
            cbor_to_json!(fil_actor_verifreg_state::v17::ExtendClaimTermsParams, bytes)
        }
        REMOVE_EXPIRED_CLAIMS | REMOVE_EXPIRED_CLAIMS_EXPORTED => {
            cbor_to_json!(
                fil_actor_verifreg_state::v17::RemoveExpiredClaimsParams,
                bytes
            )
        }
        UNIVERSAL_RECEIVER_HOOK => {
            cbor_to_json!(fil_actor_verifreg_state::v17::AllocationRequests, bytes)
        }
        CONSTRUCTOR => decode_empty_param(bytes),
        _ => bail!("Unknown verifreg method number: {method_num}"),
    }
}

pub fn decode_return(_version: ActorVersion, method_num: u64, bytes: &[u8]) -> Result<Value> {
    use methods::*;
    match method_num {
        REMOVE_VERIFIED_CLIENT_DATA_CAP => {
            cbor_to_json!(fil_actor_verifreg_state::v17::RemoveDataCapReturn, bytes)
        }
        REMOVE_EXPIRED_ALLOCATIONS | REMOVE_EXPIRED_ALLOCATIONS_EXPORTED => {
            cbor_to_json!(
                fil_actor_verifreg_state::v17::RemoveExpiredAllocationsReturn,
                bytes
            )
        }
        CLAIM_ALLOCATIONS => {
            cbor_to_json!(fil_actor_verifreg_state::v17::ClaimAllocationsReturn, bytes)
        }
        GET_CLAIMS | GET_CLAIMS_EXPORTED => {
            cbor_to_json!(fil_actor_verifreg_state::v17::GetClaimsReturn, bytes)
        }
        REMOVE_EXPIRED_CLAIMS | REMOVE_EXPIRED_CLAIMS_EXPORTED => {
            cbor_to_json!(
                fil_actor_verifreg_state::v17::RemoveExpiredClaimsReturn,
                bytes
            )
        }
        UNIVERSAL_RECEIVER_HOOK => {
            cbor_to_json!(fil_actor_verifreg_state::v17::AllocationsResponse, bytes)
        }
        // Methods with no meaningful return
        CONSTRUCTOR | ADD_VERIFIER | REMOVE_VERIFIER | ADD_VERIFIED_CLIENT
        | ADD_VERIFIED_CLIENT_EXPORTED | EXTEND_CLAIM_TERMS | EXTEND_CLAIM_TERMS_EXPORTED => {
            decode_empty_param(bytes)
        }
        _ => bail!("Return decoding not implemented for verifreg method {method_num}"),
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
    fn test_decode_remove_expired_allocations_params() {
        let params = fil_actor_verifreg_state::v17::RemoveExpiredAllocationsParams {
            client: 42,
            allocation_ids: vec![10, 20],
        };
        let cbor = to_vec(&params).unwrap();
        let result =
            decode_params(ActorVersion::V17, methods::REMOVE_EXPIRED_ALLOCATIONS, &cbor).unwrap();
        assert_eq!(result["client"], 42);
        assert_eq!(result["allocation_ids"], json!([10, 20]));
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
