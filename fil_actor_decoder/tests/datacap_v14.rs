//! Snapshot tests for Datacap (f07) actor — v14.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V14;
const A: ActorType = ActorType::DataCap;

/// TransferExported params — mainnet epoch 4457228 (NV23/Waffle era)
/// Source: bafy2bzaceaeim7hnqzr55vwcgytlsxwopsm7nzurmdqynmi5t6kzt7jd6v7wq
#[test]
fn mainnet_transfer_params() {
    let hex = "834200064d006f05b59d3b20000000000000584d8281861a00307300d82a5828000181e203922020709305f312a66323c14b35351802c5cf7361813dc794372df89e3b663f72941e1b00000008000000001a0007e9001a005033401a004451ca80";
    insta::assert_json_snapshot!(decode_params(A, V, 80475954, &decode_hex(hex)).unwrap());
}

/// TransferExported return — same message
#[test]
fn mainnet_transfer_return() {
    let hex = "834f001711ddb705f32c40000000000000500004ce3e170e007c110a0c76a30c00004d83820180820080811a04e465b4";
    insta::assert_json_snapshot!(decode_return(A, V, 80475954, &decode_hex(hex)).unwrap());
}
