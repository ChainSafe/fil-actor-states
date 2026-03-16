//! Snapshot tests for Datacap (f07) actor — v11.
//! Real on-chain data from mainnet.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V11;
const A: ActorType = ActorType::DataCap;

/// IncreaseAllowanceExported params — mainnet epoch 3019386 (NV19-20/Lightning-Thunder era)
/// Source: bafy2bzacea66krgstyo6gi6y2bbpye5d35xol6ol5e4fprlqokswkouaakxme
#[test]
fn mainnet_increase_allowance_params() {
    let hex = "8255019165a67a951b7795e5bcc88f36627218364909854d006f05b59d3b20000000000000";
    insta::assert_json_snapshot!(
        decode_params(A, V, 1777121560, &decode_hex(hex)).unwrap()
    );
}

/// IncreaseAllowanceExported return — same message
#[test]
fn mainnet_increase_allowance_return() {
    let hex = "4d006f05b59d3b20000000000000";
    insta::assert_json_snapshot!(
        decode_return(A, V, 1777121560, &decode_hex(hex)).unwrap()
    );
}
