//! Snapshot tests for Datacap (f07) actor — v17.
//!
//! Real on-chain data sourced via `Filecoin.ChainGetMessage` / `Filecoin.StateReplay`
//! against https://filfox.info/rpc/v1 (mainnet) and https://calibration.filfox.info/rpc/v1.
//! Synthetic tests use `fvm_ipld_encoding::to_vec` on actual v17 types.

use fil_actor_decoder::{ActorType, ActorVersion, decode_params, decode_return};
use fvm_ipld_encoding::to_vec;
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;

fn decode_hex(hex: &str) -> Vec<u8> {
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

const V: ActorVersion = ActorVersion::V17;
const A: ActorType = ActorType::DataCap;

// ── Real on-chain ───────────────────────────────────────────────────────

/// TransferFromExported params — mainnet epoch 5833541
/// Source: bafy2bzaceczilb2427k5qucq5gic6groqm6jvmkfh3djcuupgsnnocqnf2sgs
#[test]
fn mainnet_transfer_from_params() {
    let hex = "844500f3e1d7014200064e0003e7336287142000000000000059029d8289861a003917d5d82a5828000181e2039220209e0e0e891fbf41594df42e399568b395516573dcbb3a11c5551b8b330f5e08311b00000008000000001a0007e9001a005033401a005ba62f861a003917d5d82a5828000181e203922020abc76cc4db1d6f5839511427ce1d5cc9bfa2a74d794bc849badb32ac7b7598141b00000008000000001a0007e9001a005033401a005ba632861a003917d5d82a5828000181e2039220209de6e7c5b2d90236419d6e52283922e7da810f67ef8ee8643ba9c63e979355011b00000008000000001a0007e9001a005033401a005ba636861a003917d5d82a5828000181e203922020156e792b858648a625ea5b781be69fa27d6b11f9e2d17a90fc7821175f568a171b00000008000000001a0007e9001a005033401a005ba63a861a003917d5d82a5828000181e2039220202f92e1b2678e395f4eadacee1bea3362e4c592bfe281431d79cccfafece971311b00000008000000001a0007e9001a005033401a005ba63a861a003917d5d82a5828000181e203922020b81e3bd166def41d6a4e72d8e45ec9e3c08a4e180d911a0c82451a370a1094361b00000008000000001a0007e9001a005033401a005ba63c861a003917d5d82a5828000181e2039220203a673fad8c9a0143c822946b939fd51e9759687cad9e9da80184c7cf4765c8061b00000008000000001a0007e9001a005033401a005ba641861a003917d5d82a5828000181e203922020abaa1c7ca669da15106e8e842db9cdb51c7f0c313d5319d98cfd3e79aac183261b00000008000000001a0007e9001a005033401a005ba642861a003917d5d82a5828000181e20392202099c2daf1f95883627f570aa9197dcaf0db17cc3ba3042af26c2196754dc8381d1b00000008000000001a0007e9001a005033401a005ba64280";
    let result = decode_params(A, V, 3621052141, &decode_hex(hex)).unwrap();
    insta::assert_json_snapshot!(result);
}

/// TransferFromExported return — mainnet epoch 5833541
/// Source: bafy2bzaceczilb2427k5qucq5gic6groqm6jvmkfh3djcuupgsnnocqnf2sgs
#[test]
fn mainnet_transfer_from_return() {
    let hex = "844f002b64d8087cb645a8000000000000500008a1be0855c6267d563ac6c96800004f0018ba708c8e3c47a0000000000000583583820980820080891a0763a1e91a0763a1ea1a0763a1eb1a0763a1ec1a0763a1ed1a0763a1ee1a0763a1ef1a0763a1f01a0763a1f1";
    let result = decode_return(A, V, 3621052141, &decode_hex(hex)).unwrap();
    insta::assert_json_snapshot!(result);
}

// ── Synthetic ───────────────────────────────────────────────────────────

#[test]
fn constructor_params() {
    let p = fil_actor_datacap_state::v17::ConstructorParams {
        governor: Address::new_id(90),
    };
    insta::assert_json_snapshot!(decode_params(A, V, 1, &to_vec(&p).unwrap()).unwrap());
}

#[test]
fn mint_params() {
    let p = fil_actor_datacap_state::v17::MintParams {
        to: Address::new_id(1234),
        amount: TokenAmount::from_atto(1_000_000_000_000_000_000_i64),
        operators: vec![Address::new_id(5678)],
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Mint"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn destroy_params() {
    let p = fil_actor_datacap_state::v17::DestroyParams {
        owner: Address::new_id(100),
        amount: TokenAmount::from_atto(500),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Destroy"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn balance_params() {
    let p = fil_actor_datacap_state::v17::BalanceParams {
        address: Address::new_id(42),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Balance"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn name_return() {
    let r = fil_actor_datacap_state::v17::NameReturn {
        name: "DataCap".into(),
    };
    insta::assert_json_snapshot!(
        decode_return(
            A,
            V,
            frc42_dispatch::method_hash!("Name"),
            &to_vec(&r).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn symbol_return() {
    let r = fil_actor_datacap_state::v17::SymbolReturn {
        symbol: "DCAP".into(),
    };
    insta::assert_json_snapshot!(
        decode_return(
            A,
            V,
            frc42_dispatch::method_hash!("Symbol"),
            &to_vec(&r).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn total_supply_return() {
    let r = fil_actor_datacap_state::v17::TotalSupplyReturn {
        supply: TokenAmount::from_atto(800_000_000_000_000_000_000_000_i128),
    };
    insta::assert_json_snapshot!(
        decode_return(
            A,
            V,
            frc42_dispatch::method_hash!("TotalSupply"),
            &to_vec(&r).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn balance_return() {
    let r = fil_actor_datacap_state::v17::BalanceReturn {
        balance: TokenAmount::from_atto(5_000_000_000_000_000_000_i64),
    };
    insta::assert_json_snapshot!(
        decode_return(
            A,
            V,
            frc42_dispatch::method_hash!("Balance"),
            &to_vec(&r).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn granularity_return() {
    let r = fil_actor_datacap_state::v17::GranularityReturn {
        granularity: 1_000_000_000_000_000_000,
    };
    insta::assert_json_snapshot!(
        decode_return(
            A,
            V,
            frc42_dispatch::method_hash!("Granularity"),
            &to_vec(&r).unwrap()
        )
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
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Transfer"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn burn_params() {
    use fil_actors_shared::frc46_token::token::types::BurnParams;
    let p = BurnParams {
        amount: TokenAmount::from_atto(50_000_000_000_000_000_000_i128),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Burn"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn burn_from_params() {
    use fil_actors_shared::frc46_token::token::types::BurnFromParams;
    let p = BurnFromParams {
        owner: Address::new_id(200),
        amount: TokenAmount::from_atto(25_000_000_000_000_000_000_i128),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("BurnFrom"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn increase_allowance_params() {
    use fil_actors_shared::frc46_token::token::types::IncreaseAllowanceParams;
    let p = IncreaseAllowanceParams {
        operator: Address::new_id(300),
        increase: TokenAmount::from_atto(10_000_000_000_000_000_000_i128),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("IncreaseAllowance"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn decrease_allowance_params() {
    use fil_actors_shared::frc46_token::token::types::DecreaseAllowanceParams;
    let p = DecreaseAllowanceParams {
        operator: Address::new_id(300),
        decrease: TokenAmount::from_atto(5_000_000_000_000_000_000_i64),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("DecreaseAllowance"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn revoke_allowance_params() {
    use fil_actors_shared::frc46_token::token::types::RevokeAllowanceParams;
    let p = RevokeAllowanceParams {
        operator: Address::new_id(300),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("RevokeAllowance"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn allowance_params() {
    use fil_actors_shared::frc46_token::token::types::GetAllowanceParams;
    let p = GetAllowanceParams {
        owner: Address::new_id(100),
        operator: Address::new_id(300),
    };
    insta::assert_json_snapshot!(
        decode_params(
            A,
            V,
            frc42_dispatch::method_hash!("Allowance"),
            &to_vec(&p).unwrap()
        )
        .unwrap()
    );
}
