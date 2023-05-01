// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::from_address_v3_to_v2;
use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;

use crate::io::get_obj;
/// Account actor method.
pub type Method = fil_actor_account_v8::Method;

/// Account actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_account_v8::State),
    V9(fil_actor_account_v9::State),
    V10(fil_actor_account_v10::State),
    V11(fil_actor_account_v11::State),
}

pub fn is_v8_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .account
        .v8()
        .map_or(false, |cids| cids.contains(cid))
}

pub fn is_v9_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .account
        .v9()
        .map_or(false, |cids| cids.contains(cid))
}

pub fn is_v10_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .account
        .v10()
        .map_or(false, |cids| cids.contains(cid))
}

pub fn is_v11_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .account
        .v11()
        .map_or(false, |cids| cids.contains(cid))
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_account_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_account_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_account_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_account_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown account actor code {}", actor.code))
    }

    pub fn pubkey_address(&self) -> Address {
        match self {
            State::V8(st) => st.address,
            State::V9(st) => st.address,
            State::V10(st) => from_address_v3_to_v2(st.address),
            State::V11(st) => from_address_v3_to_v2(st.address),
        }
    }
}
