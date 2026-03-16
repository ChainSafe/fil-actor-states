//! Snapshot tests for Verified Registry (f06) actor — v14.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V14;
const A: ActorType = ActorType::VerifiedRegistry;

/// ExtendClaimTerms return — mainnet epoch 4327216 (NV23/Waffle era)
/// Source: bafy2bzacecswz3h3uqyuom3zzydx42zps5ilfjraqqhqynkladmoxrpl5tcmu
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8218c880";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}
