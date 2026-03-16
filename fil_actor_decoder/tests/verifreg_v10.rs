//! Snapshot tests for Verified Registry (f06) actor — v10.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V10;
const A: ActorType = ActorType::VerifiedRegistry;

/// AddVerifiedClient — mainnet epoch 2739863 (NV18/Hygge era)
/// Source: bafy2bzacebqmyb2xkrbw4evogqkfrbc637sd34szfmu2ylgt7hqyw57glmto6
#[test]
fn mainnet_add_verified_client_params() {
    let hex = "825501b148275e403de2fc35ab6d59e861fbc6a8ac6ffd46000800000000";
    insta::assert_json_snapshot!(decode_params(A, V, 4, &decode_hex(hex)).unwrap());
}

/// ExtendClaimTerms return — mainnet epoch 3031412 (NV18/Hygge era)
/// Source: bafy2bzacedrnt7ecmv6zrcqsyj34bsekrww53yxq5ygvx6izdojm5xzzvjxwe
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8218bc80";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}
