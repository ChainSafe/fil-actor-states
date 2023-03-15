// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use serde::Serialize;

use crate::io::get_obj;

/// Cron actor address.
pub const ADDRESS: Address = Address::new_id(3);

/// Cron actor method.
pub type Method = fil_actor_cron_v8::Method;

/// Cron actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_cron_v8::State),
    V9(fil_actor_cron_v9::State),
    V10(fil_actor_cron_v10::State),
}

pub fn is_v8_cron_cid(cid: &Cid) -> bool {
    let known_cids = [
        // calibnet v8
        Cid::try_from("bafk2bzaceaxlezmclw5ugldhhtfgvn7yztux45scqik3ez4yhwiqhg5ssib44").unwrap(),
        // mainnet v8
        Cid::try_from("bafk2bzacecqb3eolfurehny6yp7tgmapib4ocazo5ilkopjce2c7wc2bcec62").unwrap(),
        // devnet v8
        Cid::try_from("bafk2bzacecgrlf3vg3mufwovddlbgclhpnpp3jftr46stssh3crd3pyljc37w").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v9_cron_cid(cid: &Cid) -> bool {
    let known_cids = [
        // calibnet v9
        Cid::try_from("bafk2bzaceb7hxmudhvkizszbmmf2ur2qfnfxfkok3xmbrlifylx6huw4bb3s4").unwrap(),
        // mainnet v9
        Cid::try_from("bafk2bzacebcec3lffmos3nawm5cvwehssxeqwxixoyyfvejy7viszzsxzyu26").unwrap(),
        // devnet v9
        Cid::try_from("bafk2bzaceahwdt32ji53mo5yz6imvztz3s3g2ra5uz3jdfa77j7hqcnq6r4l2").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v10_cron_cid(cid: &Cid) -> bool {
    let known_cids = [
        // calibnet v10
        Cid::try_from("bafk2bzacecw2yjb6ysieffa7lk7xd32b3n4ssowvafolt7eq52lp6lk4lkhji").unwrap(),
        // mainnet v10
        Cid::try_from("bafk2bzacedcbtsifegiu432m5tysjzkxkmoczxscb6hqpmrr6img7xzdbbs2g").unwrap(),
        // devnet v10
        Cid::try_from("bafk2bzaceabslrigld2vshng6sppbp3bsptjtttvbxctwqe5lkyl2efom2wu4").unwrap(),
    ];
    known_cids.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_cron_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_cron_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_cron_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown cron actor code {}", actor.code))
    }
}
