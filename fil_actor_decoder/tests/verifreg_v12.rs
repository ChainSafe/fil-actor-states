//! Snapshot tests for Verified Registry (f06) actor — v12.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V12;
const A: ActorType = ActorType::VerifiedRegistry;

/// ExtendClaimTerms return — mainnet epoch 3569536 (NV21/Watermelon era)
/// Source: bafy2bzaceaabi66hdqakr6yqnquunfkdm7vi3qreh6chrneczezmmx4so7hq4
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8219019080";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}
