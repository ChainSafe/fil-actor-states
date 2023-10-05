// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use serde::Serialize;

use crate::io::get_obj;

/// EVM actor method.
pub type Method = fil_actor_evm_state::v10::Method;

/// EVM actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V10(fil_actor_evm_state::v10::State),
    V11(fil_actor_evm_state::v11::State),
    V12(fil_actor_evm_state::v12::State),
}

pub fn is_v10_evm_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.evm.v10.contains(cid)
}

pub fn is_v11_evm_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.evm.v11.contains(cid)
}

pub fn is_v12_evm_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.evm.v12.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v10_evm_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_evm_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_evm_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown evm actor code {}", code))
    }
}
