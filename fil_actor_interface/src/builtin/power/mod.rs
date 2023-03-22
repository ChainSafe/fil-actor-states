// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use cid::Cid;
use fil_actors_runtime_v9::runtime::Policy;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, econ::TokenAmount, sector::StoragePower};
use serde::{Deserialize, Serialize};

use crate::{io::get_obj, FilterEstimate};

/// Power actor address.
// TODO: Select address based on actors version
pub const ADDRESS: Address = Address::new_id(4);

/// Power actor method.
// TODO: Select method based on actors version
pub type Method = fil_actor_power_v8::Method;

pub fn is_v8_power_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.power.v8.contains(cid)
}

pub fn is_v9_power_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.power.v9.contains(cid)
}

pub fn is_v10_power_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.power.v10.contains(cid)
}

/// Power actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_power_v8::State),
    V9(fil_actor_power_v9::State),
    V10(fil_actor_power_v10::State),
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_power_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_power_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_power_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown power actor code {}", actor.code))
    }

    /// Consume state to return just total quality adj power
    pub fn into_total_quality_adj_power(self) -> StoragePower {
        match self {
            State::V8(st) => st.total_quality_adj_power,
            State::V9(st) => st.total_quality_adj_power,
            State::V10(st) => st.total_quality_adj_power,
        }
    }

    /// Returns the total power claim.
    pub fn total_power(&self) -> Claim {
        match self {
            State::V8(st) => Claim {
                raw_byte_power: st.total_raw_byte_power.clone(),
                quality_adj_power: st.total_quality_adj_power.clone(),
            },
            State::V9(st) => Claim {
                raw_byte_power: st.total_raw_byte_power.clone(),
                quality_adj_power: st.total_quality_adj_power.clone(),
            },
            State::V10(st) => Claim {
                raw_byte_power: st.total_raw_byte_power.clone(),
                quality_adj_power: st.total_quality_adj_power.clone(),
            },
        }
    }

    /// Consume state to return total locked funds
    pub fn into_total_locked(self) -> TokenAmount {
        match self {
            State::V8(st) => st.into_total_locked(),
            State::V9(st) => st.into_total_locked(),
            State::V10(st) => fil_utils::convert::from_token_v3_to_v2(st.into_total_locked()),
        }
    }

    /// Loads power for a given miner, if exists.
    pub fn miner_power<BS: Blockstore>(
        &self,
        s: &BS,
        miner: &Address,
    ) -> anyhow::Result<Option<Claim>> {
        match self {
            State::V8(st) => Ok(st.miner_power(&s, miner)?.map(From::from)),
            State::V9(st) => Ok(st.miner_power(&s, miner)?.map(From::from)),
            State::V10(st) => Ok(st
                .miner_power(&s, &fil_utils::convert::from_address_v2_to_v3(*miner))?
                .map(From::from)),
        }
    }

    /// Checks power actor state for if miner meets minimum consensus power.
    pub fn miner_nominal_power_meets_consensus_minimum<BS: Blockstore>(
        &self,
        policy: &Policy,
        s: &BS,
        miner: &Address,
    ) -> anyhow::Result<bool> {
        match self {
            State::V8(st) => st.miner_nominal_power_meets_consensus_minimum(policy, &s, miner),
            State::V9(st) => st.miner_nominal_power_meets_consensus_minimum(policy, &s, miner),
            State::V10(st) => st
                .miner_nominal_power_meets_consensus_minimum(policy, &s, miner.id()?)
                .map(|(_, bool_val)| bool_val)
                .map_err(|e| anyhow::anyhow!("{}", e)),
        }
    }

    /// Returns `this_epoch_qa_power_smoothed` from the state.
    pub fn total_power_smoothed(&self) -> FilterEstimate {
        match self {
            State::V8(st) => st.this_epoch_qa_power_smoothed.clone(),
            State::V9(st) => st.this_epoch_qa_power_smoothed.clone(),
            State::V10(st) => fil_utils::convert::from_filter_estimate_v3_to_v2(
                st.this_epoch_qa_power_smoothed.clone(),
            ),
        }
    }

    /// Returns total locked funds
    pub fn total_locked(&self) -> TokenAmount {
        match self {
            State::V8(st) => st.total_pledge_collateral.clone(),
            State::V9(st) => st.total_pledge_collateral.clone(),
            State::V10(st) => {
                fil_utils::convert::from_token_v3_to_v2(st.total_pledge_collateral.clone())
            }
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Claim {
    /// Sum of raw byte power for a miner's sectors.
    pub raw_byte_power: StoragePower,
    /// Sum of quality adjusted power for a miner's sectors.
    pub quality_adj_power: StoragePower,
}

impl From<fil_actor_power_v8::Claim> for Claim {
    fn from(cl: fil_actor_power_v8::Claim) -> Self {
        Self {
            raw_byte_power: cl.raw_byte_power,
            quality_adj_power: cl.quality_adj_power,
        }
    }
}

impl From<fil_actor_power_v9::Claim> for Claim {
    fn from(cl: fil_actor_power_v9::Claim) -> Self {
        Self {
            raw_byte_power: cl.raw_byte_power,
            quality_adj_power: cl.quality_adj_power,
        }
    }
}

impl From<fil_actor_power_v10::Claim> for Claim {
    fn from(cl: fil_actor_power_v10::Claim) -> Self {
        Self {
            raw_byte_power: cl.raw_byte_power,
            quality_adj_power: cl.quality_adj_power,
        }
    }
}
