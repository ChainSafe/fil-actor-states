// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;

use crate::io::get_obj;

/// System actor address.
pub const ADDRESS: Address = Address::new_id(0);

/// System actor method.
pub type Method = fil_actor_system_v8::Method;

/// System actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_system_v8::State),
    V9(fil_actor_system_v9::State),
    V10(fil_actor_system_v10::State),
    V11(fil_actor_system_v11::State),
}

pub fn is_v8_system_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.system.v8.contains(cid)
}

pub fn is_v9_system_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.system.v9.contains(cid)
}

pub fn is_v10_system_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.system.v10.contains(cid)
}

pub fn is_v11_system_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.system.v11.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_system_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_system_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_system_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_system_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown system actor code {}", actor.code))
    }
}
