// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod abi;
pub mod v10;
pub mod v11;
pub mod v12;
pub mod v8;
pub mod v9;

// Re-exports
pub extern crate fvm_ipld_amt;
pub extern crate fvm_ipld_bitfield;
pub extern crate fvm_ipld_blockstore;
pub extern crate fvm_ipld_encoding;
pub extern crate fvm_ipld_hamt;
pub extern crate fvm_shared as fvm_shared2;
pub extern crate fvm_shared3;

use fvm_ipld_hamt::BytesKey;
use fvm_shared4::ActorID;
use integer_encoding::VarInt;

// code copied from `frc46_token`
pub mod frc46_token {
    use crate::actor_id_key;
    use cid::Cid;
    use fvm_ipld_blockstore::Blockstore;
    use fvm_ipld_encoding::tuple::*;
    use fvm_ipld_hamt::Hamt;
    use fvm_ipld_hamt::{BytesKey, Error as HamtError};
    use fvm_shared3::econ::TokenAmount;
    use fvm_shared4::ActorID;

    type Map<'bs, BS, K, V> = Hamt<&'bs BS, V, K>;
    type BalanceMap<'bs, BS> = Map<'bs, BS, BytesKey, TokenAmount>;
    type AllowanceMap<'bs, BS> = Map<'bs, BS, BytesKey, Cid>;

    type Result<T> = std::result::Result<T, HamtError>;

    /// This value has been chosen to optimise to reduce gas-costs when accessing the balances map. Non-
    /// standard use cases of the token library might find a different value to be more efficient.
    pub const DEFAULT_HAMT_BIT_WIDTH: u32 = 3;

    /// An abstraction over the IPLD layer to get and modify token state without dealing with HAMTs etc.
    ///
    /// This is a simple wrapper of state and in general does not account for token protocol level
    /// checks such as ensuring necessary approvals are enforced during transfers. This is left for the
    /// caller to handle. However, some invariants such as non-negative balances, allowances and total
    /// supply are enforced.
    #[derive(Serialize_tuple, Deserialize_tuple, PartialEq, Eq, Clone, Debug)]
    pub struct TokenState {
        /// Total supply of token
        pub supply: TokenAmount,
        /// Map<ActorId, TokenAmount> of balances as a Hamt
        pub balances: Cid,
        /// Map<ActorId, Map<ActorId, TokenAmount>> as a Hamt. Allowances are stored balances[owner][operator]
        pub allowances: Cid,
        /// Bit-width to use when loading Hamts
        pub hamt_bit_width: u32,
    }

    impl TokenState {
        /// Create a new token state-tree, without committing it (the root cid) to a blockstore
        pub fn new<BS: Blockstore>(store: &BS) -> Result<Self> {
            Self::new_with_bit_width(store, DEFAULT_HAMT_BIT_WIDTH)
        }

        /// Create a new token state-tree, without committing it (the root cid) to a blockstore
        ///
        /// Explicitly sets the bit width of underlying Hamt structures. Caller must ensure
        /// 1 <= hamt_bit_width <= 8.
        pub fn new_with_bit_width<BS: Blockstore>(store: &BS, hamt_bit_width: u32) -> Result<Self> {
            // Blockstore is still needed to create valid Cids for the Hamts
            let empty_balance_map =
                BalanceMap::new_with_bit_width(store, hamt_bit_width).flush()?;
            let empty_allowances_map =
                AllowanceMap::new_with_bit_width(store, hamt_bit_width).flush()?;

            Ok(Self {
                supply: Default::default(),
                balances: empty_balance_map,
                allowances: empty_allowances_map,
                hamt_bit_width,
            })
        }

        /// Get the balance of an ActorID from the currently stored state
        pub fn get_balance<BS: Blockstore>(
            &self,
            bs: &BS,
            owner: ActorID,
        ) -> Result<Option<TokenAmount>> {
            let balances = self.get_balance_map(bs)?;
            balances
                .get(&actor_id_key(owner))
                .map(|amount_opt| amount_opt.map(|opt| opt.to_owned()))
        }

        /// Retrieve the balance map as a HAMT
        pub fn get_balance_map<'bs, BS: Blockstore>(
            &self,
            bs: &'bs BS,
        ) -> Result<BalanceMap<'bs, BS>> {
            Ok(BalanceMap::load_with_bit_width(
                &self.balances,
                bs,
                self.hamt_bit_width,
            )?)
        }
    }
}

pub fn actor_id_key(a: ActorID) -> BytesKey {
    a.encode_var_vec().into()
}
