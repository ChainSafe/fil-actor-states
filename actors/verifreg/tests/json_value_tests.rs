#![cfg(feature = "json")]

//! Unit tests for `to_json_value()` on verifreg types across all versions.
//! These test the derive macro output directly, independent of the decoder.

use cid::Cid;
use fvm_shared4::piece::PaddedPieceSize;
use serde_json::Value;

fn test_cid() -> Cid {
    "bafkqaaa".parse::<Cid>().unwrap()
}

/// Generate core verifreg tests for a version that has Allocation, Claim, and common types.
/// Covers: field names, Cid→string, PaddedPieceSize→u64, ActorID→number, BigInt→string.
macro_rules! test_verifreg_core {
    ($mod_name:ident, $version:ident) => {
        mod $mod_name {
            use super::*;
            use fil_actor_verifreg_state::$version;

            #[test]
            fn allocation_produces_named_fields() {
                let alloc = $version::Allocation {
                    client: 1001,
                    provider: 2002,
                    data: test_cid(),
                    size: PaddedPieceSize(1 << 30),
                    term_min: 518400,
                    term_max: 1555200,
                    expiration: 4000000,
                };
                let json = alloc.to_json_value();
                let obj = json.as_object().expect("expected JSON object");
                assert_eq!(obj.len(), 7);
                assert_eq!(obj["client"], serde_json::json!(1001));
                assert_eq!(obj["provider"], serde_json::json!(2002));
                assert!(obj["data"].is_string()); // Cid → string
                assert_eq!(obj["size"], serde_json::json!(1 << 30)); // PaddedPieceSize → u64
            }

            #[test]
            fn claim_produces_named_fields() {
                let claim = $version::Claim {
                    provider: 3003,
                    client: 1001,
                    data: test_cid(),
                    size: PaddedPieceSize(1 << 20),
                    term_min: 518400,
                    term_max: 1555200,
                    term_start: 100000,
                    sector: 42,
                };
                let json = claim.to_json_value();
                let obj = json.as_object().unwrap();
                assert_eq!(obj.len(), 8);
                assert_eq!(obj["sector"], serde_json::json!(42));
            }

            #[test]
            fn extend_claim_terms_nested_vec() {
                let params = $version::ExtendClaimTermsParams {
                    terms: vec![$version::ClaimTerm {
                        provider: 100,
                        claim_id: 1,
                        term_max: 999999,
                    }],
                };
                let json = params.to_json_value();
                let terms = json.as_object().unwrap()["terms"].as_array().unwrap();
                assert_eq!(terms.len(), 1);
                let t0 = terms[0].as_object().unwrap();
                assert_eq!(t0["provider"], serde_json::json!(100));
            }
        }
    };
}

/// Test BigInt fields (VerifierParams.allowance has #[serde(with = "bigint_ser")])
macro_rules! test_verifreg_bigint {
    ($mod_name:ident, $version:ident) => {
        mod $mod_name {
            use super::*;
            use fil_actor_verifreg_state::$version;
            use fvm_shared4::bigint::BigInt;

            #[test]
            fn verifier_params_bigint_field() {
                let params = $version::VerifierParams {
                    address: fvm_shared4::address::Address::new_id(555),
                    allowance: BigInt::from(1_000_000_000_000i64),
                };
                let json = params.to_json_value();
                let obj = json.as_object().unwrap();
                assert_eq!(obj["address"], Value::String("f0555".into()));
                assert_eq!(obj["allowance"], Value::String("1000000000000".into()));
            }

            #[test]
            fn sector_claim_summary_transparent_bigint() {
                let summary = $version::SectorClaimSummary {
                    claimed_space: BigInt::from(34359738368i64),
                };
                assert_eq!(summary.to_json_value(), Value::String("34359738368".into()));
            }
        }
    };
}

/// Test BatchReturn integration (nested in AllocationsResponse)
macro_rules! test_verifreg_batch_return {
    ($mod_name:ident, $version:ident, $shared_version:ident) => {
        mod $mod_name {
            use fil_actor_verifreg_state::$version;
            use fil_actors_shared::$shared_version::BatchReturn;

            #[test]
            fn batch_return_nested_in_response() {
                let resp = $version::AllocationsResponse {
                    allocation_results: BatchReturn::ok(3),
                    extension_results: BatchReturn::ok(0),
                    new_allocations: vec![10, 11, 12],
                };
                let json = resp.to_json_value();
                let obj = json.as_object().unwrap();
                let alloc_results = obj["allocation_results"].as_object().unwrap();
                assert_eq!(alloc_results["success_count"], serde_json::json!(3));
                assert!(alloc_results["fail_codes"].as_array().unwrap().is_empty());
                let allocs = obj["new_allocations"].as_array().unwrap();
                assert_eq!(
                    allocs,
                    &[
                        serde_json::json!(10),
                        serde_json::json!(11),
                        serde_json::json!(12)
                    ]
                );
            }
        }
    };
}

// v9-v11 use fvm_shared v2/v3 PaddedPieceSize — tested via decoder snapshot tests.
// v12+ use fvm_shared4 types and can be tested directly here.
test_verifreg_core!(v12_core, v12);
test_verifreg_core!(v13_core, v13);
test_verifreg_core!(v14_core, v14);
test_verifreg_core!(v15_core, v15);
test_verifreg_core!(v16_core, v16);
test_verifreg_core!(v17_core, v17);

// BigInt tests only for v12+ (v9-v11 VerifierParams use different fvm_shared Address types)
test_verifreg_bigint!(v12_bigint, v12);
test_verifreg_bigint!(v13_bigint, v13);
test_verifreg_bigint!(v14_bigint, v14);
test_verifreg_bigint!(v15_bigint, v15);
test_verifreg_bigint!(v16_bigint, v16);
test_verifreg_bigint!(v17_bigint, v17);

// BatchReturn tests — v9-v11 use fvm_shared v2/v3, tested via decoder snapshots.
test_verifreg_batch_return!(v12_batch, v12, v12);
test_verifreg_batch_return!(v13_batch, v13, v13);
test_verifreg_batch_return!(v14_batch, v14, v14);
test_verifreg_batch_return!(v15_batch, v15, v15);
test_verifreg_batch_return!(v16_batch, v16, v16);
test_verifreg_batch_return!(v17_batch, v17, v17);

/// Cross-version equivalence for v12-v17 (all CBOR-identical)
#[test]
fn v12_through_v17_allocation_produce_same_output() {
    let cid = test_cid();
    let expected = fil_actor_verifreg_state::v17::Allocation {
        client: 1001,
        provider: 2002,
        data: cid,
        size: PaddedPieceSize(1 << 30),
        term_min: 518400,
        term_max: 1555200,
        expiration: 4000000,
    }
    .to_json_value();

    macro_rules! assert_alloc_eq {
        ($ver:ident) => {
            let a = fil_actor_verifreg_state::$ver::Allocation {
                client: 1001,
                provider: 2002,
                data: cid,
                size: PaddedPieceSize(1 << 30),
                term_min: 518400,
                term_max: 1555200,
                expiration: 4000000,
            };
            assert_eq!(a.to_json_value(), expected, "{} mismatch", stringify!($ver));
        };
    }
    assert_alloc_eq!(v12);
    assert_alloc_eq!(v13);
    assert_alloc_eq!(v14);
    assert_alloc_eq!(v15);
    assert_alloc_eq!(v16);
}
