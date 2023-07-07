// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::strict_bytes;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared3::address::Address;

pub mod account {
    use super::*;

    pub const AUTHENTICATE_MESSAGE_METHOD: u64 = frc42_macros::method_hash!("AuthenticateMessage");

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct AuthenticateMessageParams {
        #[serde(with = "strict_bytes")]
        pub signature: Vec<u8>,
        #[serde(with = "strict_bytes")]
        pub message: Vec<u8>,
    }
}

pub mod datacap {
    use super::*;
    use fvm_shared3::econ::TokenAmount;

    #[repr(u64)]
    pub enum Method {
        Mint = frc42_macros::method_hash!("Mint"),
        Destroy = frc42_macros::method_hash!("Destroy"),
        Balance = frc42_macros::method_hash!("Balance"),
        Transfer = frc42_macros::method_hash!("Transfer"),
        Burn = frc42_macros::method_hash!("Burn"),
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
    pub struct MintParams {
        pub to: Address,
        pub amount: TokenAmount,
        pub operators: Vec<Address>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
    pub struct DestroyParams {
        pub owner: Address,
        pub amount: TokenAmount,
    }
}
