// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::from_token_v3_to_v2;
use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount};
use serde::Serialize;

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
    V12(fil_actor_multisig_state::v11::State),
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
        Err(anyhow::anyhow!("Unknown multisig actor code {}", code))
    }

    /// Returns amount locked in multisig contract
    pub fn locked_balance(&self, height: ChainEpoch) -> anyhow::Result<TokenAmount> {
        Ok(match self {
            State::V8(st) => st.amount_locked(height),
            State::V9(st) => st.amount_locked(height),
            State::V10(st) => from_token_v3_to_v2(st.amount_locked(height)),
            State::V11(st) => from_token_v3_to_v2(st.amount_locked(height)),
            State::V12(st) => from_token_v3_to_v2(st.amount_locked(height)), // Why not `from_token_v4_to_v2` here?
        })
    }
}
