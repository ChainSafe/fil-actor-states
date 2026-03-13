#![cfg(feature = "json")]

use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;
use serde_json::Value;

/// Test that v16 and v17 datacap types produce identical JSON structure,
/// verifying the derive macro works consistently across actor versions.
mod v16_tests {
    use super::*;
    use fil_actor_datacap_state::v16;

    #[test]
    fn mint_params_produces_named_fields() {
        let params = v16::MintParams {
            to: Address::new_id(1234),
            amount: TokenAmount::from_atto(5_000_000),
            operators: vec![Address::new_id(5678), Address::new_id(9999)],
        };

        let json = params.to_json_value();
        assert!(json.is_object(), "expected JSON object, got: {json}");

        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert!(obj.contains_key("to"));
        assert!(obj.contains_key("amount"));
        assert!(obj.contains_key("operators"));

        // Address → string
        assert_eq!(obj["to"], Value::String("f01234".to_string()));
        // TokenAmount → atto string
        assert_eq!(obj["amount"], Value::String("5000000".to_string()));
        // Vec<Address> → array of strings
        let ops = obj["operators"].as_array().unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], Value::String("f05678".to_string()));
    }

    #[test]
    fn destroy_params_produces_named_fields() {
        let params = v16::DestroyParams {
            owner: Address::new_id(42),
            amount: TokenAmount::from_atto(999),
        };

        let json = params.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["owner"], Value::String("f042".to_string()));
        assert_eq!(obj["amount"], Value::String("999".to_string()));
    }

    #[test]
    fn transparent_types_produce_inner_value() {
        // ConstructorParams is #[serde(transparent)] with a single Address field
        let params = v16::ConstructorParams {
            governor: Address::new_id(100),
        };
        let json = params.to_json_value();
        assert_eq!(json, Value::String("f0100".to_string()));

        // GranularityReturn is transparent with a u64 field
        let ret = v16::GranularityReturn { granularity: 42 };
        let json = ret.to_json_value();
        assert_eq!(json, serde_json::json!(42));

        // NameReturn is transparent with a String field
        let ret = v16::NameReturn {
            name: "DataCap".to_string(),
        };
        let json = ret.to_json_value();
        assert_eq!(json, Value::String("DataCap".to_string()));
    }

    #[test]
    fn token_amount_transparent_types() {
        let ret = v16::TotalSupplyReturn {
            supply: TokenAmount::from_atto(1_000_000_000),
        };
        let json = ret.to_json_value();
        assert_eq!(json, Value::String("1000000000".to_string()));

        let ret = v16::BalanceReturn {
            balance: TokenAmount::from_atto(0),
        };
        let json = ret.to_json_value();
        assert_eq!(json, Value::String("0".to_string()));
    }
}

mod v17_tests {
    use super::*;
    use fil_actor_datacap_state::v17;

    #[test]
    fn mint_params_produces_named_fields() {
        let params = v17::MintParams {
            to: Address::new_id(1234),
            amount: TokenAmount::from_atto(5_000_000),
            operators: vec![Address::new_id(5678), Address::new_id(9999)],
        };

        let json = params.to_json_value();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert_eq!(obj["to"], Value::String("f01234".to_string()));
        assert_eq!(obj["amount"], Value::String("5000000".to_string()));

        let ops = obj["operators"].as_array().unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], Value::String("f05678".to_string()));
        assert_eq!(ops[1], Value::String("f09999".to_string()));
    }

    #[test]
    fn transparent_types_produce_inner_value() {
        let params = v17::ConstructorParams {
            governor: Address::new_id(100),
        };
        let json = params.to_json_value();
        assert_eq!(json, Value::String("f0100".to_string()));
    }

    #[test]
    fn v16_and_v17_produce_same_output() {
        let v16_params = fil_actor_datacap_state::v16::MintParams {
            to: Address::new_id(1234),
            amount: TokenAmount::from_atto(5_000_000),
            operators: vec![Address::new_id(5678)],
        };
        let v17_params = v17::MintParams {
            to: Address::new_id(1234),
            amount: TokenAmount::from_atto(5_000_000),
            operators: vec![Address::new_id(5678)],
        };

        assert_eq!(v16_params.to_json_value(), v17_params.to_json_value());
    }
}
