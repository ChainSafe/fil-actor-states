//! Snapshot tests for Datacap (f07) actor — v13.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V13;
const A: ActorType = ActorType::DataCap;

/// TransferExported params — mainnet epoch 4144893 (NV22/Dragon era)
/// Source: bafy2bzacebaju3i4rgt7yn7fdts2tceqsvisazpwac3xjxngh2an3zhegibkq
#[test]
fn mainnet_transfer_params() {
    let hex = "834200064e00022b1c8c1227a00000000000005901758285861a00303cadd82a5828000181e2039220202e9391a1b9b26f2fc2ec602406442fb176631bbe46d1ea1275d799626f8f79281b00000008000000001a0007e9001a000bdd801a003f7727861a00303cadd82a5828000181e20392202014f852a16257ce81754b7175e5fdef22dce537f5ac8c750e958fad61893017211b00000008000000001a0007e9001a000bdd801a003f772b861a00303cadd82a5828000181e203922020f07a7552f2be0818d0986d0c3f13e67ac6603884f3cfc5aeaa6981bbc0065d071b00000008000000001a0007e9001a000bdd801a003f7733861a00303cadd82a5828000181e2039220206d6cf39f355251c29b9f7f0b208470163402fb87ed33b022d67e5a506016f9211b00000008000000001a0007e9001a000bdd801a003f7737861a00303cadd82a5828000181e203922020b788d8d28dde9d2cd6f18a6b14bfd2ae56a9975a53e545941cdfcb30e5eb18031b00000008000000001a0007e9001a000bdd801a003f773780";
    insta::assert_json_snapshot!(decode_params(A, V, 80475954, &decode_hex(hex)).unwrap());
}

/// TransferExported return — same message
#[test]
fn mainnet_transfer_return() {
    let hex = "834f0028f2110ceed03b400000000000005000048cc325867a0dc71011395e200000582183820580820080851a043495f51a043495f61a043495f71a043495f81a043495f9";
    insta::assert_json_snapshot!(decode_return(A, V, 80475954, &decode_hex(hex)).unwrap());
}
