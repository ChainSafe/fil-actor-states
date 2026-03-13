#![cfg(feature = "json")]

use cid::Cid;
use fvm_shared4::address::Address;
use fvm_shared4::piece::PaddedPieceSize;
use serde_json::Value;

fn test_cid() -> Cid {
    // A well-known empty identity CID (bafkqaaa)
    "bafkqaaa".parse::<Cid>().unwrap()
}

/// Tests for v16 verifreg types
mod v16_tests {
    use super::*;
    use fil_actor_verifreg_state::v16;

    #[test]
    fn allocation_produces_named_fields() {
        let alloc = v16::Allocation {
            client: 1001,
            provider: 2002,
            data: test_cid(),
            size: PaddedPieceSize(1 << 30), // 1 GiB
            term_min: 518400,
            term_max: 1555200,
            expiration: 4000000,
        };

        let json = alloc.to_json_value();
        assert!(json.is_object(), "expected JSON object, got: {json}");

        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 7);
        assert_eq!(obj["client"], serde_json::json!(1001));
        assert_eq!(obj["provider"], serde_json::json!(2002));
        assert!(obj["data"].is_string()); // Cid -> string
        assert_eq!(obj["size"], serde_json::json!(1 << 30)); // PaddedPieceSize -> inner u64
        assert_eq!(obj["term_min"], serde_json::json!(518400));
        assert_eq!(obj["term_max"], serde_json::json!(1555200));
        assert_eq!(obj["expiration"], serde_json::json!(4000000));
    }

    #[test]
    fn claim_produces_named_fields() {
        let claim = v16::Claim {
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
        assert_eq!(obj["provider"], serde_json::json!(3003));
        assert_eq!(obj["client"], serde_json::json!(1001));
        assert_eq!(obj["sector"], serde_json::json!(42));
        assert_eq!(obj["term_start"], serde_json::json!(100000));
    }

    #[test]
    fn verifier_params_bigint_field() {
        use fvm_shared4::bigint::BigInt;

        let params = v16::VerifierParams {
            address: Address::new_id(555),
            allowance: BigInt::from(1_000_000_000_000i64),
        };

        let json = params.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["address"], Value::String("f0555".to_string()));
        // BigInt with #[serde(with = "bigint_ser")] -> auto-detected as string
        assert_eq!(
            obj["allowance"],
            Value::String("1000000000000".to_string())
        );
    }

    #[test]
    fn nested_structs_with_vec() {
        let params = v16::ExtendClaimTermsParams {
            terms: vec![
                v16::ClaimTerm {
                    provider: 100,
                    claim_id: 1,
                    term_max: 999999,
                },
                v16::ClaimTerm {
                    provider: 200,
                    claim_id: 2,
                    term_max: 888888,
                },
            ],
        };

        let json = params.to_json_value();
        let obj = json.as_object().unwrap();
        let terms = obj["terms"].as_array().unwrap();
        assert_eq!(terms.len(), 2);

        let t0 = terms[0].as_object().unwrap();
        assert_eq!(t0["provider"], serde_json::json!(100));
        assert_eq!(t0["claim_id"], serde_json::json!(1));
        assert_eq!(t0["term_max"], serde_json::json!(999999));
    }

    #[test]
    fn transparent_constructor_params() {
        let params = v16::ConstructorParams {
            root_key: Address::new_id(99),
        };
        let json = params.to_json_value();
        assert_eq!(json, Value::String("f099".to_string()));
    }

    #[test]
    fn batch_return_nested_in_response() {
        use fil_actors_shared::v16::BatchReturn;

        let resp = v16::AllocationsResponse {
            allocation_results: BatchReturn::ok(3),
            extension_results: BatchReturn::ok(0),
            new_allocations: vec![10, 11, 12],
        };

        let json = resp.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 3);

        // BatchReturn itself has to_json_value() producing named fields
        let alloc_results = obj["allocation_results"].as_object().unwrap();
        assert_eq!(alloc_results["success_count"], serde_json::json!(3));
        assert!(alloc_results["fail_codes"].as_array().unwrap().is_empty());

        // Vec<AllocationID> -> array of numbers
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

    #[test]
    fn allocation_claim_with_cid_and_padded_piece_size() {
        let claim = v16::AllocationClaim {
            client: 42,
            allocation_id: 7,
            data: test_cid(),
            size: PaddedPieceSize(512),
        };

        let json = claim.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["client"], serde_json::json!(42));
        assert_eq!(obj["allocation_id"], serde_json::json!(7));
        assert!(obj["data"].as_str().unwrap().starts_with("bafk"));
        assert_eq!(obj["size"], serde_json::json!(512));
    }

    #[test]
    fn sector_claim_summary_transparent_bigint() {
        use fvm_shared4::bigint::BigInt;

        let summary = v16::SectorClaimSummary {
            claimed_space: BigInt::from(34359738368i64), // 32 GiB
        };
        let json = summary.to_json_value();
        // transparent + bigint_ser -> string
        assert_eq!(json, Value::String("34359738368".to_string()));
    }
}

/// Tests for v17 verifreg types -- verify same output as v16
mod v17_tests {
    use super::*;
    use fil_actor_verifreg_state::v17;

    #[test]
    fn allocation_produces_named_fields() {
        let alloc = v17::Allocation {
            client: 1001,
            provider: 2002,
            data: test_cid(),
            size: PaddedPieceSize(1 << 30),
            term_min: 518400,
            term_max: 1555200,
            expiration: 4000000,
        };

        let json = alloc.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 7);
        assert_eq!(obj["client"], serde_json::json!(1001));
        assert_eq!(obj["provider"], serde_json::json!(2002));
        assert_eq!(obj["size"], serde_json::json!(1 << 30));
    }

    #[test]
    fn v16_and_v17_allocation_produce_same_output() {
        let cid = test_cid();
        let v16_alloc = fil_actor_verifreg_state::v16::Allocation {
            client: 1001,
            provider: 2002,
            data: cid,
            size: PaddedPieceSize(1 << 30),
            term_min: 518400,
            term_max: 1555200,
            expiration: 4000000,
        };
        let v17_alloc = v17::Allocation {
            client: 1001,
            provider: 2002,
            data: cid,
            size: PaddedPieceSize(1 << 30),
            term_min: 518400,
            term_max: 1555200,
            expiration: 4000000,
        };

        assert_eq!(v16_alloc.to_json_value(), v17_alloc.to_json_value());
    }

    #[test]
    fn v16_and_v17_claim_produce_same_output() {
        let cid = test_cid();
        let v16_claim = fil_actor_verifreg_state::v16::Claim {
            provider: 3003,
            client: 1001,
            data: cid,
            size: PaddedPieceSize(1 << 20),
            term_min: 518400,
            term_max: 1555200,
            term_start: 100000,
            sector: 42,
        };
        let v17_claim = v17::Claim {
            provider: 3003,
            client: 1001,
            data: cid,
            size: PaddedPieceSize(1 << 20),
            term_min: 518400,
            term_max: 1555200,
            term_start: 100000,
            sector: 42,
        };

        assert_eq!(v16_claim.to_json_value(), v17_claim.to_json_value());
    }

    #[test]
    fn nested_claim_allocations_params() {
        let params = v17::ClaimAllocationsParams {
            sectors: vec![v17::SectorAllocationClaims {
                sector: 7,
                expiry: 5000000,
                claims: vec![v17::AllocationClaim {
                    client: 42,
                    allocation_id: 1,
                    data: test_cid(),
                    size: PaddedPieceSize(512),
                }],
            }],
            all_or_nothing: true,
        };

        let json = params.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["all_or_nothing"], serde_json::json!(true));

        let sectors = obj["sectors"].as_array().unwrap();
        assert_eq!(sectors.len(), 1);
        let s0 = sectors[0].as_object().unwrap();
        assert_eq!(s0["sector"], serde_json::json!(7));
        assert_eq!(s0["expiry"], serde_json::json!(5000000));

        let claims = s0["claims"].as_array().unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(
            claims[0].as_object().unwrap()["client"],
            serde_json::json!(42)
        );
    }
}
