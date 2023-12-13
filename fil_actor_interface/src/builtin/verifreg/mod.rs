// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;
use crate::io::get_obj;

/// verifreg actor address.
pub const ADDRESS: Address = Address::new_id(6);

/// Verifreg actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_verifreg_state::v8::State),
    V9(fil_actor_verifreg_state::v9::State),
    V10(fil_actor_verifreg_state::v10::State),
    V11(fil_actor_verifreg_state::v11::State),
    V12(fil_actor_verifreg_state::v12::State),
}

pub fn is_v8_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v8.contains(cid)
}

pub fn is_v9_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v9.contains(cid)
}

pub fn is_v10_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v10.contains(cid)
}

pub fn is_v11_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v11.contains(cid)
}

pub fn is_v12_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v12.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
        where
            BS: Blockstore,
    {
        if is_v8_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown verifreg actor code {}", code))
    }
}
