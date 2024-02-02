// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{
    from_address_v3_to_v2, from_address_v4_to_v2, from_token_v3_to_v2, from_token_v4_to_v2,
};
use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount, MethodNum};
use serde::{Deserialize, Serialize};

use crate::io::get_obj;

/// Multisig actor method.
pub type Method = fil_actor_multisig_state::v8::Method;

/// Multisig actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_multisig_state::v8::State),
    V9(fil_actor_multisig_state::v9::State),
    V10(fil_actor_multisig_state::v10::State),
    V11(fil_actor_multisig_state::v11::State),
    V12(fil_actor_multisig_state::v12::State),
    V13(fil_actor_multisig_state::v13::State),
}

/// Transaction type used in multisig actor
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Transaction {
    pub id: i64,
    pub to: Address,
    pub value: TokenAmount,
    pub method: MethodNum,
    pub params: RawBytes,
    pub approved: Vec<Address>,
}

pub fn is_v8_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.multisig.v8.contains(cid)
}

pub fn is_v9_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.multisig.v9.contains(cid)
}

pub fn is_v10_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.multisig.v10.contains(cid)
}

pub fn is_v11_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.multisig.v11.contains(cid)
}

pub fn is_v12_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.multisig.v12.contains(cid)
}

pub fn is_v13_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.multisig.v13.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_multisig_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_multisig_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_multisig_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_multisig_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_multisig_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        if is_v13_multisig_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V13)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown multisig actor code {}", code))
    }

    /// Returns amount locked in multisig contract
    pub fn locked_balance(&self, height: ChainEpoch) -> anyhow::Result<TokenAmount> {
        Ok(match self {
            State::V8(st) => st.amount_locked(height),
            State::V9(st) => st.amount_locked(height),
            State::V10(st) => from_token_v3_to_v2(st.amount_locked(height)),
            State::V11(st) => from_token_v3_to_v2(st.amount_locked(height)),
            State::V12(st) => from_token_v4_to_v2(st.amount_locked(height)),
            State::V13(st) => from_token_v4_to_v2(st.amount_locked(height)),
        })
    }

    /// Returns pending transactions for the given multisig wallet
    pub fn get_pending_txn<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<Transaction>> {
        let mut res = Vec::new();
        match self {
            State::V8(st) => {
                let txns = fil_actors_shared::v8::make_map_with_root::<
                    BS,
                    fil_actor_multisig_state::v8::Transaction,
                >(&st.pending_txs, store)?;
                crate::parse_pending_transactions!(res, txns);
                Ok(res)
            }
            State::V9(st) => {
                let txns = fil_actors_shared::v9::make_map_with_root::<
                    BS,
                    fil_actor_multisig_state::v9::Transaction,
                >(&st.pending_txs, store)?;
                crate::parse_pending_transactions!(res, txns);
                Ok(res)
            }
            State::V10(st) => {
                let txns = fil_actors_shared::v10::make_map_with_root::<
                    BS,
                    fil_actor_multisig_state::v10::Transaction,
                >(&st.pending_txs, store)?;
                crate::parse_pending_transactions_v3!(res, txns);
                Ok(res)
            }
            State::V11(st) => {
                let txns = fil_actors_shared::v11::make_map_with_root::<
                    BS,
                    fil_actor_multisig_state::v11::Transaction,
                >(&st.pending_txs, store)?;
                crate::parse_pending_transactions_v3!(res, txns);
                Ok(res)
            }
            State::V12(st) => {
                let txns = fil_actor_multisig_state::v12::PendingTxnMap::load(
                    store,
                    &st.pending_txs,
                    fil_actor_multisig_state::v12::PENDING_TXN_CONFIG,
                    "pending txns",
                )
                .expect("Could not load pending transactions");
                crate::parse_pending_transactions_v4!(res, txns);
                Ok(res)
            }
            State::V13(st) => {
                let txns = fil_actor_multisig_state::v13::PendingTxnMap::load(
                    store,
                    &st.pending_txs,
                    fil_actor_multisig_state::v13::PENDING_TXN_CONFIG,
                    "pending txns",
                )
                .expect("Could not load pending transactions");
                crate::parse_pending_transactions_v4!(res, txns);
                Ok(res)
            }
        }
    }
}
