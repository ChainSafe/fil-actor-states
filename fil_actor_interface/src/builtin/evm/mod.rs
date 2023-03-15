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
    let known_cids = vec![
        // calibnet v10
        Cid::try_from("bafk2bzaceccmwmnb42pn7y7skbjwjur7b2eqxuw4lvm3he2xpvudjzluss4os").unwrap(),
        // mainnet v10
        Cid::try_from("bafk2bzaceahmzdxhqsm7cu2mexusjp6frm7r4kdesvti3etv5evfqboos2j4g").unwrap(),
        // devnet v10
        Cid::try_from("bafk2bzacec5ywczgg73fnwi36nlxso3zduop3fwj3pq6ynn5zltrs4dpcwglg").unwrap(),
    ];
    known_cids.contains(cid)
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
