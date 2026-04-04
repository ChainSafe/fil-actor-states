//! Snapshot tests for Datacap (f07) actor — v15.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V15;
const A: ActorType = ActorType::DataCap;

/// TransferFromExported params — mainnet epoch 5421192 (NV24/TukTuk era)
/// Source: bafy2bzaceasbrxgllo7evw4kigtfk45guj2vqonn6rb3vczaeynkrfyhrjz26
#[test]
fn mainnet_transfer_from_params() {
    let hex = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mainnet_datacap_v15_transfer_from_params.hex"
    ))
    .unwrap();
    insta::assert_json_snapshot!(decode_params(A, V, 3621052141, &decode_hex(hex.trim())).unwrap());
}

/// TransferFromExported return — same message
#[test]
fn mainnet_transfer_from_return() {
    let hex = "844f0008e79e12517f937c37937e080000500006c4291bbe996fba4cbecfdb3000005000095e658e0c3abae943c7f400000000585383820f80820080891a05306e471a05306e481a05306e491a05306e4a1a05306e4b1a05306e4c1a05306e4d1a05306e4e1a05306e4f1a05306e501a05306e511a05306e521a05306e531a05306e541a05306e55";
    insta::assert_json_snapshot!(decode_return(A, V, 3621052141, &decode_hex(hex)).unwrap());
}
