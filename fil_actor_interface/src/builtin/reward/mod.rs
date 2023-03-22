// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, econ::TokenAmount};
use serde::Serialize;

use crate::io::get_obj;

/// Reward actor address.
pub const ADDRESS: Address = Address::new_id(2);

/// Reward actor method.
pub type Method = fil_actor_reward_v8::Method;

pub fn is_v8_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.reward.v8.contains(cid)
}

pub fn is_v9_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.reward.v9.contains(cid)
}

pub fn is_v10_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.reward.v10.contains(cid)
}

/// Reward actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_reward_v8::State),
    V9(fil_actor_reward_v9::State),
    V10(fil_actor_reward_v10::State),
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_reward_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_reward_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_reward_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown reward actor code {}", actor.code))
    }

    /// Consume state to return just storage power reward
    pub fn into_total_storage_power_reward(self) -> TokenAmount {
        match self {
            State::V8(st) => st.into_total_storage_power_reward(),
            State::V9(st) => st.into_total_storage_power_reward(),
            State::V10(st) => {
                fil_utils::convert::from_token_v3_to_v2(st.into_total_storage_power_reward())
            }
        }
    }
}
