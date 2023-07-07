// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod abi;
pub mod v10;
pub mod v11;
pub mod v8;
pub mod v9;

pub mod frc46_token {
    use cid::Cid;
    use fvm_ipld_encoding::tuple::*;
    use fvm_shared3::econ::TokenAmount;

    #[derive(Serialize_tuple, Deserialize_tuple, PartialEq, Eq, Clone, Debug)]
    pub struct TokenState {
        /// Total supply of token
        pub supply: TokenAmount,
        /// Map<ActorId, TokenAmount> of balances as a Hamt
        pub balances: Cid,
        /// Map<ActorId, Map<ActorId, TokenAmount>> as a Hamt. Allowances are stored balances[owner][operator]
        pub allowances: Cid,
        /// Bit-width to use when loading Hamts
        hamt_bit_width: u32,
    }
}
