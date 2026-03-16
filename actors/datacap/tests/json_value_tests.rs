#![cfg(feature = "json")]

//! Unit tests for `to_json_value()` on datacap types across all versions.
//! These test the derive macro output directly, independent of the decoder.

use serde_json::Value;

/// Generate tests for a datacap version that has MintParams and DestroyParams.
/// All versions (v9-v17) have these two types.
macro_rules! test_mint_destroy {
    ($mod_name:ident, $version:ident, $addr_new_id:path, $token_from_atto:path) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn mint_params_produces_named_fields() {
                let params = fil_actor_datacap_state::$version::MintParams {
                    to: $addr_new_id(1234),
                    amount: $token_from_atto(5_000_000),
                    operators: vec![$addr_new_id(5678), $addr_new_id(9999)],
                };
                let json = params.to_json_value();
                let obj = json.as_object().expect("expected JSON object");
                assert_eq!(obj.len(), 3);
                assert!(obj.contains_key("to"));
                assert!(obj.contains_key("amount"));
                assert!(obj.contains_key("operators"));
                assert_eq!(obj["to"], Value::String("f01234".into()));
                assert_eq!(obj["amount"], Value::String("5000000".into()));
                let ops = obj["operators"].as_array().unwrap();
                assert_eq!(ops.len(), 2);
                assert_eq!(ops[0], Value::String("f05678".into()));
            }

            #[test]
            fn destroy_params_produces_named_fields() {
                let params = fil_actor_datacap_state::$version::DestroyParams {
                    owner: $addr_new_id(42),
                    amount: $token_from_atto(999),
                };
                let json = params.to_json_value();
                let obj = json.as_object().unwrap();
                assert_eq!(obj["owner"], Value::String("f042".into()));
                assert_eq!(obj["amount"], Value::String("999".into()));
            }
        }
    };
}

/// Generate tests for versions that have the full type set (v12-v17).
/// Adds transparent type tests and cross-version equivalence.
macro_rules! test_full_types {
    ($mod_name:ident, $version:ident) => {
        mod $mod_name {
            use super::*;
            use fil_actor_datacap_state::$version;
            use fvm_shared4::address::Address;
            use fvm_shared4::econ::TokenAmount;

            #[test]
            fn transparent_constructor() {
                let p = $version::ConstructorParams {
                    governor: Address::new_id(100),
                };
                assert_eq!(p.to_json_value(), Value::String("f0100".into()));
            }

            #[test]
            fn transparent_name_return() {
                let r = $version::NameReturn {
                    name: "DataCap".into(),
                };
                assert_eq!(r.to_json_value(), Value::String("DataCap".into()));
            }

            #[test]
            fn transparent_granularity_return() {
                let r = $version::GranularityReturn { granularity: 42 };
                assert_eq!(r.to_json_value(), serde_json::json!(42));
            }

            #[test]
            fn transparent_token_amount_types() {
                let r = $version::TotalSupplyReturn {
                    supply: TokenAmount::from_atto(1_000_000_000),
                };
                assert_eq!(r.to_json_value(), Value::String("1000000000".into()));

                let r = $version::BalanceReturn {
                    balance: TokenAmount::from_atto(0),
                };
                assert_eq!(r.to_json_value(), Value::String("0".into()));
            }
        }
    };
}

// v9-v11 use fvm_shared v2/v3 types — tested via decoder snapshot tests.
// v12+ use fvm_shared4 types and can be tested directly here.
test_mint_destroy!(
    v12,
    v12,
    fvm_shared4::address::Address::new_id,
    fvm_shared4::econ::TokenAmount::from_atto
);
test_mint_destroy!(
    v13,
    v13,
    fvm_shared4::address::Address::new_id,
    fvm_shared4::econ::TokenAmount::from_atto
);
test_mint_destroy!(
    v14,
    v14,
    fvm_shared4::address::Address::new_id,
    fvm_shared4::econ::TokenAmount::from_atto
);
test_mint_destroy!(
    v15,
    v15,
    fvm_shared4::address::Address::new_id,
    fvm_shared4::econ::TokenAmount::from_atto
);
test_mint_destroy!(
    v16,
    v16,
    fvm_shared4::address::Address::new_id,
    fvm_shared4::econ::TokenAmount::from_atto
);
test_mint_destroy!(
    v17,
    v17,
    fvm_shared4::address::Address::new_id,
    fvm_shared4::econ::TokenAmount::from_atto
);

// v11 has full types but uses fvm_shared3 — can't construct with fvm_shared4 types.
// Full type tests for v12-v17 (fvm_shared4, have ConstructorParams, NameReturn, etc.)
test_full_types!(v12_full, v12);
test_full_types!(v13_full, v13);
test_full_types!(v14_full, v14);
test_full_types!(v15_full, v15);
test_full_types!(v16_full, v16);
test_full_types!(v17_full, v17);

/// Cross-version equivalence: all versions produce the same JSON for MintParams.
#[test]
fn all_versions_produce_same_mint_output() {
    use fvm_shared4::address::Address;
    use fvm_shared4::econ::TokenAmount;

    let expected = fil_actor_datacap_state::v17::MintParams {
        to: Address::new_id(1234),
        amount: TokenAmount::from_atto(5_000_000),
        operators: vec![Address::new_id(5678)],
    }
    .to_json_value();

    // v12-v16 use fvm_shared4 — construct directly
    macro_rules! assert_mint_eq {
        ($ver:ident) => {
            let p = fil_actor_datacap_state::$ver::MintParams {
                to: Address::new_id(1234),
                amount: TokenAmount::from_atto(5_000_000),
                operators: vec![Address::new_id(5678)],
            };
            assert_eq!(
                p.to_json_value(),
                expected,
                "v{} mismatch",
                stringify!($ver)
            );
        };
    }
    assert_mint_eq!(v12);
    assert_mint_eq!(v13);
    assert_mint_eq!(v14);
    assert_mint_eq!(v15);
    assert_mint_eq!(v16);
}
