// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::known_cids::INIT_V0_ACTOR_CID;
use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;

use crate::io::get_obj;

/// Init actor address.
pub const ADDRESS: Address = Address::new_id(1);

/// Init actor method.
pub type Method = fil_actor_init_state::v8::Method;

pub fn is_v0_init_cid(cid: &Cid) -> bool {
    cid == &*INIT_V0_ACTOR_CID
}

pub fn is_v8_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v8.contains(cid)
}

pub fn is_v9_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v9.contains(cid)
}

pub fn is_v10_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v10.contains(cid)
}

pub fn is_v11_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v11.contains(cid)
}

pub fn is_v12_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v12.contains(cid)
}

pub fn is_v13_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v13.contains(cid)
}

pub fn is_v14_init_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.init.v14.contains(cid)
}

/// Init actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V0(fil_actor_init_state::v0::State),
    V8(fil_actor_init_state::v8::State),
    V9(fil_actor_init_state::v9::State),
    V10(fil_actor_init_state::v10::State),
    V11(fil_actor_init_state::v11::State),
    V12(fil_actor_init_state::v12::State),
    V13(fil_actor_init_state::v13::State),
    V14(fil_actor_init_state::v14::State),
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v0_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V0)
                .context("Actor state doesn't exist in store");
        }
        if is_v8_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        if is_v13_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V13)
                .context("Actor state doesn't exist in store");
        }
        if is_v14_init_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V14)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown init actor code {}", code))
    }

    pub fn into_network_name(self) -> String {
        match self {
            State::V0(st) => st.network_name,
            State::V8(st) => st.network_name,
            State::V9(st) => st.network_name,
            State::V10(st) => st.network_name,
            State::V11(st) => st.network_name,
            State::V12(st) => st.network_name,
            State::V13(st) => st.network_name,
            State::V14(st) => st.network_name,
        }
    }
}
