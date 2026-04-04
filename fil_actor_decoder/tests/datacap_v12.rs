//! Snapshot tests for Datacap (f07) actor — v12.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V12;
const A: ActorType = ActorType::DataCap;

/// TransferExported params — mainnet epoch 3922481 (NV21/Watermelon era)
/// Source: bafy2bzacea3llgubzhqznvb3pdhg56wcfoa6p7ne3x7jm3lm3okgjefyyok5o
#[test]
fn mainnet_transfer_params() {
    let hex = "834200064e0001bc16d674ec8000000000000059012b8284861a002f0688d82a5828000181e203922020b999522c47d73f46df363dee85bb3076cbfd5cf651085413d661db4c1ac544351b00000008000000001a00100a401a005033401a003c77af861a002f0688d82a5828000181e20392202076c8ddc46d1f83dadec5ff6d3479904e6e4ba79ac02f5d5bfe1f6535cd03b2031b00000008000000001a00100a401a005033401a003c77af861a002f0688d82a5828000181e203922020754691b2103f9c6d0ac93b8cb453a2698153494154273eefbfdfc1208e2664051b00000008000000001a00100a401a005033401a003c77af861a002f0688d82a5828000181e203922020cbf0afb40c61555a4490613c5d338c2a6bb424d9c1b482b96f777a739369ec2c1b00000008000000001a00100a401a005033401a003c77af80";
    insta::assert_json_snapshot!(decode_params(A, V, 80475954, &decode_hex(hex)).unwrap());
}

/// TransferExported return — same message
#[test]
fn mainnet_transfer_return() {
    let hex = "834f0036735ed883156bc000000000000050000501e970dca062a4c2990000000000581c83820480820080841a03ca69321a03ca69331a03ca69341a03ca6935";
    insta::assert_json_snapshot!(decode_return(A, V, 80475954, &decode_hex(hex)).unwrap());
}
