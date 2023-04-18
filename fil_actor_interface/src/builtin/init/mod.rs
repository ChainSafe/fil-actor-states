// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;

use crate::io::get_obj;

/// Init actor address.
pub const ADDRESS: Address = Address::new_id(1);

/// Init actor method.
pub type Method = fil_actor_init_v8::Method;

pub fn is_v8_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.init.v8.contains(cid)
}

pub fn is_v9_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.init.v9.contains(cid)
}

pub fn is_v10_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.init.v10.contains(cid)
}

pub fn is_v11_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.init.v11.contains(cid)
}

/// Init actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_init_v8::State),
    V9(fil_actor_init_v9::State),
    V10(fil_actor_init_v10::State),
    V11(fil_actor_init_v11::State),
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_init_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_init_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_init_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_init_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown init actor code {}", actor.code))
    }

    pub fn into_network_name(self) -> String {
        match self {
            State::V8(st) => st.network_name,
            State::V9(st) => st.network_name,
            State::V10(st) => st.network_name,
            State::V11(st) => st.network_name,
        }
    }
}
