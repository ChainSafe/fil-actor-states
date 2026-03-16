//! Snapshot tests for Verified Registry (f06) actor — v17.
//!
//! Real on-chain data sourced via `Filecoin.ChainGetMessage` / `Filecoin.StateReplay`
//! against https://filfox.info/rpc/v1 (mainnet) and https://calibration.filfox.info/rpc/v1.
//! Synthetic tests use `fvm_ipld_encoding::to_vec` on actual v17 types.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};
use fvm_ipld_encoding::to_vec;
use fvm_shared4::address::Address;
use num_bigint::BigInt;

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V17;
const A: ActorType = ActorType::VerifiedRegistry;

// ── Real on-chain — mainnet ─────────────────────────────────────────────

/// AddVerifiedClient params — mainnet epoch 5844742
/// Source: bafy2bzacebr2jvqzsqr6bxzll3zxl5f3rt74ovxnrynotcxqfgtmsumthbjmw
#[test]
fn mainnet_add_verified_client_params() {
    let hex = "825501eb50a2528a325eadc4bb68d975a4d1700c9eeaa3480001900000000000";
    insta::assert_json_snapshot!(decode_params(A, V, 4, &decode_hex(hex)).unwrap());
}

/// RemoveExpiredAllocations params — mainnet epoch 5682849
/// Source: bafy2bzacebf6jykld5fl4lkmbtgjw54mumprjowyzbzoevesjqqpy2w7jczaq
#[test]
fn mainnet_remove_expired_allocations_params() {
    let hex = "821a00303d2980";
    insta::assert_json_snapshot!(decode_params(A, V, 8, &decode_hex(hex)).unwrap());
}

/// RemoveExpiredAllocations return — mainnet epoch 5682849 (8537 bytes, loaded from fixture)
/// Source: bafy2bzacebf6jykld5fl4lkmbtgjw54mumprjowyzbzoevesjqqpy2w7jczaq
#[test]
fn mainnet_remove_expired_allocations_return() {
    let hex = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mainnet_verifreg_remove_expired_allocations_return.hex"
    ))
    .unwrap();
    insta::assert_json_snapshot!(decode_return(A, V, 8, &decode_hex(hex.trim())).unwrap());
}

/// ExtendClaimTerms return — mainnet epoch 5844995 (300 claims, all successful)
/// Source: bafy2bzacedu4zc7w3dt5lthct7gog3hsfbtw2ervtgcxm6yfot23be25tvbbs
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8219012c80";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}

// ── Real on-chain — calibnet ────────────────────────────────────────────

/// AddVerifiedClient params — calibnet epoch 3531235
/// Source: bafy2bzacebnkn7rm4hohnpxwpx3s3rjr3k2jhcixdgf753sq5rbqmtcqkvdcg
#[test]
fn calibnet_add_verified_client_params() {
    let hex = "8258310387624255a0765266b5759ca7775e4bd8b03ff786699696f99203d71440aec01b5b6351cdf92bf891858eda927512bc81470015d3ef798000";
    insta::assert_json_snapshot!(decode_params(A, V, 4, &decode_hex(hex)).unwrap());
}

/// AddVerifiedClientExported params — calibnet epoch 3531856
/// Source: bafy2bzacedthu6mzx4iqopahgreqc4ujz6hlg235jgr7mixaqeh2camtbzzdw
#[test]
fn calibnet_add_verified_client_exported_params() {
    let hex = "8244009bfc094400100000";
    insta::assert_json_snapshot!(decode_params(A, V, 3916220144, &decode_hex(hex)).unwrap());
}

// ── Synthetic ───────────────────────────────────────────────────────────

#[test]
fn remove_verifier_params() {
    let p = fil_actor_verifreg_state::v17::RemoveVerifierParams {
        verifier: Address::new_id(999),
    };
    insta::assert_json_snapshot!(decode_params(A, V, 3, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn get_claims_params() {
    let p = fil_actor_verifreg_state::v17::GetClaimsParams {
        provider: 1234,
        claim_ids: vec![100, 200, 300],
    };
    insta::assert_json_snapshot!(decode_params(A, V, 10, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn get_claims_return() {
    let r = fil_actor_verifreg_state::v17::GetClaimsReturn {
        batch_info: fil_actors_shared::v17::BatchReturn {
            success_count: 3,
            fail_codes: vec![],
        },
        claims: vec![fil_actor_verifreg_state::v17::Claim {
            provider: 1234,
            client: 5678,
            data: cid::Cid::default(),
            size: fvm_shared4::piece::PaddedPieceSize(34359738368),
            term_min: 518400,
            term_max: 5256000,
            term_start: 4000000,
            sector: 42,
        }],
    };
    insta::assert_json_snapshot!(decode_return(A, V, 10, &to_vec(&r).unwrap()).unwrap());
}

#[test]
fn claim_allocations_params() {
    let p = fil_actor_verifreg_state::v17::ClaimAllocationsParams {
        sectors: vec![fil_actor_verifreg_state::v17::SectorAllocationClaims {
            sector: 100,
            expiry: 6000000,
            claims: vec![fil_actor_verifreg_state::v17::AllocationClaim {
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
fn claim_allocations_return() {
    let r = fil_actor_verifreg_state::v17::ClaimAllocationsReturn {
        sector_results: fil_actors_shared::v17::BatchReturn {
            success_count: 1,
            fail_codes: vec![],
        },
        sector_claims: vec![fil_actor_verifreg_state::v17::SectorClaimSummary {
            claimed_space: BigInt::from(34359738368_u64),
        }],
    };
    insta::assert_json_snapshot!(decode_return(A, V, 9, &to_vec(&r).unwrap()).unwrap());
}

#[test]
fn extend_claim_terms_params() {
    let p = fil_actor_verifreg_state::v17::ExtendClaimTermsParams {
        terms: vec![fil_actor_verifreg_state::v17::ClaimTerm {
            provider: 1234,
            claim_id: 5678,
            term_max: 5256000,
        }],
    };
    insta::assert_json_snapshot!(decode_params(A, V, 11, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn remove_expired_claims_params() {
    let p = fil_actor_verifreg_state::v17::RemoveExpiredClaimsParams {
        provider: 1234,
        claim_ids: vec![10, 20, 30],
    };
    insta::assert_json_snapshot!(decode_params(A, V, 12, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn remove_expired_claims_return() {
    let r = fil_actor_verifreg_state::v17::RemoveExpiredClaimsReturn {
        considered: vec![10, 20, 30],
        results: fil_actors_shared::v17::BatchReturn {
            success_count: 2,
            fail_codes: vec![fil_actors_shared::v17::FailCode {
                idx: 2,
                code: fvm_shared4::error::ExitCode::USR_NOT_FOUND,
            }],
        },
    };
    insta::assert_json_snapshot!(decode_return(A, V, 12, &to_vec(&r).unwrap()).unwrap());
}

#[test]
fn universal_receiver_hook_params() {
    let p = fil_actor_verifreg_state::v17::AllocationRequests {
        allocations: vec![fil_actor_verifreg_state::v17::AllocationRequest {
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
        decode_params(A, V, frc42_dispatch::method_hash!("Receive"), &to_vec(&p).unwrap()).unwrap()
    );
}
