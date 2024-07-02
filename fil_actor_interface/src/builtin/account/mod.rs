// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{from_address_v3_to_v2, from_address_v4_to_v2};
use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;

use crate::io::get_obj;
/// Account actor method.
pub type Method = fil_actor_account_state::v8::Method;

/// Account actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_account_state::v8::State),
    V9(fil_actor_account_state::v9::State),
    V10(fil_actor_account_state::v10::State),
    V11(fil_actor_account_state::v11::State),
    V12(fil_actor_account_state::v12::State),
    V13(fil_actor_account_state::v13::State),
    V14(fil_actor_account_state::v14::State),
}

pub fn is_v8_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v8.contains(cid)
}

pub fn is_v9_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v9.contains(cid)
}

pub fn is_v10_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v10.contains(cid)
}

pub fn is_v11_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v11.contains(cid)
}

pub fn is_v12_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v12.contains(cid)
}

pub fn is_v13_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v13.contains(cid)
}

pub fn is_v14_account_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.account.v14.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        if is_v13_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V13)
                .context("Actor state doesn't exist in store");
        }
        if is_v14_account_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V14)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown account actor code {}", code))
    }

    pub fn pubkey_address(&self) -> Address {
        match self {
            State::V8(st) => st.address,
            State::V9(st) => st.address,
            State::V10(st) => from_address_v3_to_v2(st.address),
            State::V11(st) => from_address_v3_to_v2(st.address),
            State::V12(st) => from_address_v4_to_v2(st.address),
            State::V13(st) => from_address_v4_to_v2(st.address),
            State::V14(st) => from_address_v4_to_v2(st.address),
        }
    }
}
