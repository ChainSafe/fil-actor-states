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
    let known_cids = vec![
        // calibnet v8
        Cid::try_from("bafk2bzaceadyfilb22bcvzvnpzbg2lyg6npmperyq6es2brvzjdh5rmywc4ry").unwrap(),
        // mainnet
        Cid::try_from("bafk2bzaceaipvjhoxmtofsnv3aj6gj5ida4afdrxa4ewku2hfipdlxpaektlw").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v9_init_cid(cid: &Cid) -> bool {
    let known_cids = vec![
        // calibnet v9
        Cid::try_from("bafk2bzaceczqxpivlxifdo5ohr2rx5ny4uyvssm6tkf7am357xm47x472yxu2").unwrap(),
        // mainnet v9
        Cid::try_from("bafk2bzacebtdq4zyuxk2fzbdkva6kc4mx75mkbfmldplfntayhbl5wkqou33i").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v10_init_cid(cid: &Cid) -> bool {
    let known_cids = vec![
        // calibnet v10
        Cid::try_from("bafk2bzacedhxbcglnonzruxf2jpczara73eh735wf2kznatx2u4gsuhgqwffq").unwrap(),
        // mainnet v10
        Cid::try_from("bafk2bzaced2f5rhir3hbpqbz5ght7ohv2kgj42g5ykxrypuo2opxsup3ykwl6").unwrap(),
    ];
    known_cids.contains(cid)
}

/// Init actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_init_v8::State),
    V9(fil_actor_init_v9::State),
    V10(fil_actor_init_v10::State),
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
        Err(anyhow::anyhow!("Unknown init actor code {}", actor.code))
    }

    pub fn into_network_name(self) -> String {
        match self {
            State::V8(st) => st.network_name,
            State::V9(st) => st.network_name,
            State::V10(st) => st.network_name,
        }
    }
}
