// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use serde::Serialize;

use crate::io::get_obj;

/// EVM actor method.
pub type Method = fil_actor_evm_v10::Method;

/// EVM actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V10(fil_actor_evm_v10::State),
}

pub fn is_v10_evm_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .evm
        .v10()
        .map_or(false, |cids| cids.contains(cid))
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v10_evm_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown evm actor code {}", actor.code))
    }
}
