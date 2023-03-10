// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use serde::Serialize;

use crate::io::get_obj;

/// Multisig actor method.
pub type Method = fil_actor_multisig_v8::Method;

/// Multisig actor state.
#[derive(Serialize)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_multisig_v8::State),
    V9(fil_actor_multisig_v9::State),
    V10(fil_actor_multisig_v10::State),
}

pub fn is_v8_multisig_cid(cid: &Cid) -> bool {
    let known_cids = vec![
        // calibnet v8
        Cid::try_from("bafk2bzacec66wmb4kohuzvuxsulhcgiwju7sqkldwfpmmgw7dbbwgm5l2574q").unwrap(),
        // mainnet v8
        Cid::try_from("bafk2bzacebhldfjuy4o5v7amrhp5p2gzv2qo5275jut4adnbyp56fxkwy5fag").unwrap(),
        // devnet v8
        Cid::try_from("bafk2bzaced4gcxjwy6garxwfw6y5a2k4jewj4t5nzopjy4qwnimhjtnsgo3ss").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v9_multisig_cid(cid: &Cid) -> bool {
    let known_cids = vec![
        // calibnet v9
        Cid::try_from("bafk2bzacec6gmi7ucukr3bk67akaxwngohw3lsg3obvdazhmfhdzflkszk3tg").unwrap(),
        // mainnet v9
        Cid::try_from("bafk2bzacec4va3nmugyqjqrs3lqyr2ij67jhjia5frvx7omnh2isha6abxzya").unwrap(),
        // devnet v9
        Cid::try_from("bafk2bzacebeiygkjupkpfxcrsidci4bvn6afkvx4lsj3ut3ywhsj654pzfgk4").unwrap(),
    ];
    known_cids.contains(cid)
}

pub fn is_v10_multisig_cid(cid: &Cid) -> bool {
    let known_cids = vec![
        // calibnet v10
        Cid::try_from("bafk2bzacebv5gdlte2pyovmz6s37me6x2rixaa6a33w6lgqdohmycl23snvwm").unwrap(),
        // mainnet v10
        Cid::try_from("bafk2bzaceduf3hayh63jnl4z2knxv7cnrdenoubni22fxersc4octlwpxpmy4").unwrap(),
        // devnet v10
        Cid::try_from("bafk2bzacectxa2izvpaybmmpvearekrybxtglctwnexzzneyn6xrnrmectmpa").unwrap(),
    ];
    known_cids.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_multisig_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_multisig_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_multisig_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!(
            "Unknown multisig actor code {}",
            actor.code
        ))
    }
}
