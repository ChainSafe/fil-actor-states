// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod abi;
pub mod v10;
pub mod v11;
pub mod v8;
pub mod v9;

// Re-exports
pub extern crate fvm_ipld_amt;
pub extern crate fvm_ipld_bitfield;
pub extern crate fvm_ipld_hamt;

// code copied from `frc46_token`
pub mod frc46_token {
    use cid::Cid;
    use fvm_ipld_blockstore::Blockstore;
    use fvm_ipld_encoding::tuple::*;
    use fvm_ipld_hamt::Hamt;
    use fvm_ipld_hamt::{BytesKey, Error as HamtError};
    use fvm_shared3::econ::TokenAmount;

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
        hamt_bit_width: u32,
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
    }
}
