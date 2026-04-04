//! Snapshot tests for Verified Registry (f06) actor — v9.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V9;
const A: ActorType = ActorType::VerifiedRegistry;

/// AddVerifiedClient — mainnet epoch 2459914 (NV17/Shark era)
/// Source: bafy2bzacecnu3o2dhjardiwmudwvjwgbpqktk3mcwfb46w4g27a4c3hlr5zce
#[test]
fn mainnet_add_verified_client_params() {
    let hex = "825501d148e4224e0f2b5cb5dc8b669015225260b1708546000800000000";
    insta::assert_json_snapshot!(decode_params(A, V, 4, &decode_hex(hex)).unwrap());
}
