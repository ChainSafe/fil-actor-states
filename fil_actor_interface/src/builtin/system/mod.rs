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
}

pub fn is_v8_system_cid(cid: &Cid) -> bool {
    let known_cids = [
        // calibnet v8
        Cid::try_from("bafk2bzaceaqrkllksxv2jsfgjvmuewx5vbzrammw5mdscod6gkdr3ijih2q64").unwrap(),
        // mainnet v8
        Cid::try_from("bafk2bzacedwq5uppsw7vp55zpj7jdieizirmldceehu6wvombw3ixq2tcq57w").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v9_system_cid(cid: &Cid) -> bool {
    let known_cids = [
        // calibnet v9
        Cid::try_from("bafk2bzaceaue3nzucbom3tcclgyaahy3iwvbqejsxrohiquakvvsjgbw3shac").unwrap(),
        // mainnet v9
        Cid::try_from("bafk2bzaceagvlo2jtahj7dloshrmwfulrd6e2izqev32qm46eumf754weec6c").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v10_system_cid(cid: &Cid) -> bool {
    let known_cids = [
        // calibnet v10
        Cid::try_from("bafk2bzacea4mtukm5zazygkdbgdf26cpnwwif5n2no7s6tknpxlwy6fpq3mug").unwrap(),
        // mainnet v10
        Cid::try_from("bafk2bzacedakk5nofebyup4m7nvx6djksfwhnxzrfuq4oyemhpl4lllaikr64").unwrap(),
    ];
    known_cids.contains(cid)
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
        Err(anyhow::anyhow!("Unknown system actor code {}", actor.code))
    }
}
