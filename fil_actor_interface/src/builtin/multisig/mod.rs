// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use serde::Serialize;

use crate::io::get_obj;

/// Multisig actor method.
pub type Method = fil_actor_multisig_v8::Method;

/// Multisig actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_multisig_v8::State),
    V9(fil_actor_multisig_v9::State),
    V10(fil_actor_multisig_v10::State),
}

pub fn is_v8_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .multisig
        .v8
        .as_ref()
        .map_or(false, |cids| cids.contains(cid))
}

pub fn is_v9_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .multisig
        .v9
        .as_ref()
        .map_or(false, |cids| cids.contains(cid))
}

pub fn is_v10_multisig_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .multisig
        .v10
        .as_ref()
        .map_or(false, |cids| cids.contains(cid))
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_multisig_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_multisig_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_multisig_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!(
            "Unknown multisig actor code {}",
            actor.code
        ))
    }
}
