//! Snapshot tests for Verified Registry (f06) actor — v11.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V11;
const A: ActorType = ActorType::VerifiedRegistry;

/// AddVerifiedClient — mainnet epoch 2990400 (NV19-20/Lightning-Thunder era)
/// Source: bafy2bzacecahd7l23crhpn2ten2ydflnis4v5pnceo7pk44hu3myoilatnfic
#[test]
fn mainnet_add_verified_client_params() {
    let hex = "82583103a982a967fec40da82e201fc04a03b9de5494bb58958a2f41fbdc59f45913e7365adafde8349c639303fbc8fcd066f13846000800000000";
    insta::assert_json_snapshot!(decode_params(A, V, 4, &decode_hex(hex)).unwrap());
}

/// ExtendClaimTerms return — mainnet epoch 3246923 (NV19-20 era)
/// Source: bafy2bzaced7tg3gbeq7ohhzrk4dq42knyctwbqkwf6xieki7e35x7ajijspna
#[test]
fn mainnet_extend_claim_terms_return() {
    let hex = "8219013580";
    insta::assert_json_snapshot!(decode_return(A, V, 11, &decode_hex(hex)).unwrap());
}
