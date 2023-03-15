// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use serde::Serialize;

use crate::io::get_obj;

/// Datacap actor method.
pub type Method = fil_actor_datacap_v10::Method;

/// Datacap actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V10(fil_actor_datacap_v10::State),
}

pub fn is_v10_datacap_cid(cid: &Cid) -> bool {
    let known_cids = vec![
        // calibnet v10
        Cid::try_from("bafk2bzaceaot6tv6p4cat3cg5fknq22htosw3p5rwyijmdsraatwqyc4qyero").unwrap(),
        // mainnet v10
        Cid::try_from("bafk2bzacealj5uk7wixhvk7l5tnredtelralwnceafqq34nb2lbylhtuyo64u").unwrap(),
    ];
    known_cids.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v10_datacap_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown datacap actor code {}", actor.code))
    }
}
