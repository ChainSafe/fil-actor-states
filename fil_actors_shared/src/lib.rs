// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod abi;
pub mod v10;
pub mod v11;
pub mod v12;
pub mod v13;
pub mod v14;
pub mod v15;
pub mod v16;
pub mod v8;
pub mod v9;

// Re-exports
pub extern crate cid;
pub extern crate filecoin_proofs_api;
pub extern crate frc46_token;
pub extern crate fvm_ipld_amt;
pub extern crate fvm_ipld_bitfield;
pub extern crate fvm_ipld_blockstore;
pub extern crate fvm_ipld_encoding;
pub extern crate fvm_ipld_hamt;
pub extern crate fvm_shared as fvm_shared2;
pub extern crate fvm_shared3;
pub extern crate fvm_shared4;
pub extern crate multihash_codetable;

pub mod ext {
    use frc46_token::token::state::{actor_id_key, StateError, TokenState};
    use fvm_ipld_blockstore::Blockstore;
    use fvm_shared4::econ::TokenAmount;
    use fvm_shared4::ActorID;

    type Result<T> = std::result::Result<T, StateError>;

    /// Lotus compatibility layer.
    pub trait TokenStateExt {
        fn get_balance_opt<BS: Blockstore>(
            &self,
            bs: &BS,
            owner: ActorID,
        ) -> Result<Option<TokenAmount>>;
    }

    impl TokenStateExt for TokenState {
        fn get_balance_opt<BS: Blockstore>(
            &self,
            bs: &BS,
            owner: ActorID,
        ) -> Result<Option<TokenAmount>> {
            let balances = self.get_balance_map(bs)?;
            Ok(balances
                .get(&actor_id_key(owner))?
                .map(|amount_opt| amount_opt.to_owned()))
        }
    }
}
