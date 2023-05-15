// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use serde::Serialize;

use crate::io::get_obj;

/// Datacap actor method.
pub type Method = fil_actor_datacap_state::v10::Method;

/// Datacap actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V10(fil_actor_datacap_state::v10::State),
    V11(fil_actor_datacap_state::v11::State),
}

pub fn is_v10_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v10.contains(cid)
}

pub fn is_v11_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v11.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v10_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown datacap actor code {}", code))
    }
}
