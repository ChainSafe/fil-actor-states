//! Snapshot tests for Verified Registry (f06) actor — v15.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V15;
const A: ActorType = ActorType::VerifiedRegistry;

/// ExtendClaimTerms return — mainnet epoch 4684953 (NV24/TukTuk era)
/// Source: bafy2bzaced3hjakmiv4olqvemztxpnmrms3ilu3pbxakm2ykel4wufcn43nl2
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8219025880";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}
