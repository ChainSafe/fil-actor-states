//! Snapshot tests for Datacap (f07) actor — v16.
//!
//! v16 types are structurally identical to v17. These tests confirm v16
//! decoding produces the same output by constructing params from v16 types.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};
use fvm_ipld_encoding::to_vec;
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;

const V: ActorVersion = ActorVersion::V16;
const A: ActorType = ActorType::DataCap;

#[test]
fn mint_params() {
    let p = fil_actor_datacap_state::v16::MintParams {
        to: Address::new_id(1234),
        amount: TokenAmount::from_atto(1_000_000_000_000_000_000_i64),
        operators: vec![Address::new_id(5678)],
    };
    insta::assert_json_snapshot!(
        decode_params(A, V, frc42_dispatch::method_hash!("Mint"), &to_vec(&p).unwrap()).unwrap()
    );
}

#[test]
fn destroy_params() {
    let p = fil_actor_datacap_state::v16::DestroyParams {
        owner: Address::new_id(100),
        amount: TokenAmount::from_atto(500),
    };
    insta::assert_json_snapshot!(
        decode_params(A, V, frc42_dispatch::method_hash!("Destroy"), &to_vec(&p).unwrap()).unwrap()
    );
}

#[test]
fn balance_params() {
    let p = fil_actor_datacap_state::v16::BalanceParams {
        address: Address::new_id(42),
    };
    insta::assert_json_snapshot!(
        decode_params(A, V, frc42_dispatch::method_hash!("Balance"), &to_vec(&p).unwrap()).unwrap()
    );
}

#[test]
fn name_return() {
    let r = fil_actor_datacap_state::v16::NameReturn { name: "DataCap".into() };
    insta::assert_json_snapshot!(
        decode_return(A, V, frc42_dispatch::method_hash!("Name"), &to_vec(&r).unwrap()).unwrap()
    );
}

#[test]
fn granularity_return() {
    let r = fil_actor_datacap_state::v16::GranularityReturn {
        granularity: 1_000_000_000_000_000_000,
    };
    insta::assert_json_snapshot!(
        decode_return(A, V, frc42_dispatch::method_hash!("Granularity"), &to_vec(&r).unwrap())
            .unwrap()
    );
}

#[test]
fn transfer_params() {
    use fil_actors_shared::frc46_token::token::types::TransferParams;
    use fvm_ipld_encoding::RawBytes;
    let p = TransferParams {
        to: Address::new_id(1000),
        amount: TokenAmount::from_atto(100_000_000_000_000_000_000_i128),
        operator_data: RawBytes::default(),
    };
    insta::assert_json_snapshot!(
        decode_params(A, V, frc42_dispatch::method_hash!("Transfer"), &to_vec(&p).unwrap())
            .unwrap()
    );
}

#[test]
fn burn_params() {
    use fil_actors_shared::frc46_token::token::types::BurnParams;
    let p = BurnParams { amount: TokenAmount::from_atto(50_000_000_000_000_000_000_i128) };
    insta::assert_json_snapshot!(
        decode_params(A, V, frc42_dispatch::method_hash!("Burn"), &to_vec(&p).unwrap()).unwrap()
    );
}
