//! Snapshot tests for Verified Registry (f06) actor — v13.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V13;
const A: ActorType = ActorType::VerifiedRegistry;

/// ExtendClaimTerms return — mainnet epoch 3953869 (NV22/Dragon era)
/// Source: bafy2bzaceary67mehfgij5p7ovneqtdh5yvh7yuiwmdktfltmnzbcs3ll7mu2
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8218c880";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}
