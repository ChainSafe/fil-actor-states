//! Snapshot tests for Verified Registry (f06) actor — v16.
//!
//! v16 types are structurally identical to v17. These tests confirm v16
//! decoding produces the same output by constructing params from v16 types.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};
use fvm_ipld_encoding::to_vec;
use fvm_shared4::address::Address;
use num_bigint::BigInt;

const V: ActorVersion = ActorVersion::V16;
const A: ActorType = ActorType::VerifiedRegistry;

#[test]
fn add_verified_client_params() {
    let p = fil_actor_verifreg_state::v16::VerifierParams {
        address: Address::new_id(1000),
        allowance: BigInt::from(5_000_000),
    };
    insta::assert_json_snapshot!(decode_params(A, V, 4, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn remove_expired_allocations_params() {
    let p = fil_actor_verifreg_state::v16::RemoveExpiredAllocationsParams {
        client: 42,
        allocation_ids: vec![10, 20],
    };
    insta::assert_json_snapshot!(decode_params(A, V, 8, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn get_claims_params() {
    let p = fil_actor_verifreg_state::v16::GetClaimsParams {
        provider: 1234,
        claim_ids: vec![100, 200, 300],
    };
    insta::assert_json_snapshot!(decode_params(A, V, 10, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn claim_allocations_params() {
    let p = fil_actor_verifreg_state::v16::ClaimAllocationsParams {
        sectors: vec![fil_actor_verifreg_state::v16::SectorAllocationClaims {
            sector: 100,
            expiry: 6000000,
            claims: vec![fil_actor_verifreg_state::v16::AllocationClaim {
                client: 5678,
                allocation_id: 9999,
                data: cid::Cid::default(),
                size: fvm_shared4::piece::PaddedPieceSize(34359738368),
            }],
        }],
        all_or_nothing: true,
    };
    insta::assert_json_snapshot!(decode_params(A, V, 9, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn extend_claim_terms_params() {
    let p = fil_actor_verifreg_state::v16::ExtendClaimTermsParams {
        terms: vec![fil_actor_verifreg_state::v16::ClaimTerm {
            provider: 1234,
            claim_id: 5678,
            term_max: 5256000,
        }],
    };
    insta::assert_json_snapshot!(decode_params(A, V, 11, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn remove_expired_claims_return() {
    let r = fil_actor_verifreg_state::v16::RemoveExpiredClaimsReturn {
        considered: vec![10, 20, 30],
        results: fil_actors_shared::v16::BatchReturn {
            success_count: 2,
            fail_codes: vec![fil_actors_shared::v16::FailCode {
                idx: 2,
                code: fvm_shared4::error::ExitCode::USR_NOT_FOUND,
            }],
        },
    };
    insta::assert_json_snapshot!(decode_return(A, V, 12, &to_vec(&r).unwrap()).unwrap());
}

#[test]
fn universal_receiver_hook_params() {
    let p = fil_actor_verifreg_state::v16::AllocationRequests {
        allocations: vec![fil_actor_verifreg_state::v16::AllocationRequest {
            provider: 1234,
            data: cid::Cid::default(),
            size: fvm_shared4::piece::PaddedPieceSize(1048576),
            term_min: 518400,
            term_max: 1036800,
            expiration: 6000000,
        }],
        extensions: vec![],
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Receive"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}
