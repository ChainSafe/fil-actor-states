// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{from_token_v3_to_v2, from_token_v4_to_v2};
use crate::io::get_obj;
use anyhow::Context;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::sector::StoragePower;
use fvm_shared::{address::Address, econ::TokenAmount, piece::PaddedPieceSize, TOTAL_FILECOIN};
use num::bigint::Sign;
use num::traits::Euclid;
use num::BigInt;
use serde::Serialize;
use std::convert::Into;
use std::ops::Mul;

/// Reward actor address.
pub const ADDRESS: Address = Address::new_id(2);

/// Reward actor method.
pub type Method = fil_actor_reward_state::v8::Method;

pub fn is_v8_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.reward.v8.contains(cid)
}

pub fn is_v9_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.reward.v9.contains(cid)
}

pub fn is_v10_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.reward.v10.contains(cid)
}

pub fn is_v11_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.reward.v11.contains(cid)
}

pub fn is_v12_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.reward.v12.contains(cid)
}

pub fn is_v13_reward_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.reward.v13.contains(cid)
}

/// Reward actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_reward_state::v8::State),
    V9(fil_actor_reward_state::v9::State),
    V10(fil_actor_reward_state::v10::State),
    V11(fil_actor_reward_state::v11::State),
    V12(fil_actor_reward_state::v12::State),
    V13(fil_actor_reward_state::v13::State),
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_reward_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_reward_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_reward_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_reward_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_reward_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        if is_v13_reward_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V13)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown reward actor code {}", code))
    }

    /// Consume state to return just storage power reward
    pub fn into_total_storage_power_reward(self) -> TokenAmount {
        match self {
            State::V8(st) => st.into_total_storage_power_reward(),
            State::V9(st) => st.into_total_storage_power_reward(),
            State::V10(st) => from_token_v3_to_v2(st.into_total_storage_power_reward()),
            State::V11(st) => from_token_v3_to_v2(st.into_total_storage_power_reward()),
            State::V12(st) => from_token_v4_to_v2(st.into_total_storage_power_reward()),
            State::V13(st) => from_token_v4_to_v2(st.into_total_storage_power_reward()),
        }
    }

    /// The baseline power the network is targeting at this state's epoch.
    pub fn this_epoch_baseline_power(self) -> StoragePower {
        match self {
            State::V8(st) => st.this_epoch_baseline_power,
            State::V9(st) => st.this_epoch_baseline_power,
            State::V10(st) => st.this_epoch_baseline_power,
            State::V11(st) => st.this_epoch_baseline_power,
            State::V12(st) => st.this_epoch_baseline_power,
            State::V13(st) => st.this_epoch_baseline_power,
        }
    }

    pub fn deal_provider_collateral_bounds(
        size: PaddedPieceSize,
        raw_byte_power: StoragePower,
        baseline_power: StoragePower,
        network_circulating_supply: TokenAmount,
    ) -> (TokenAmount, TokenAmount) {
        // The percentage of normalized circulating supply that must be covered by provider
        // collateral in a deal.
        // See: https://github.com/filecoin-project/go-state-types/blob/master/builtin/v12/market/policy.go#L9-L14.
        // Note that as of v13 the logic is exactly the same (copy-paste) for all the versions.
        // Should that change - the code here is going to need some adjustment.
        let numerator = BigInt::new(Sign::Plus, vec![1]);
        let denominator = BigInt::new(Sign::Plus, vec![100]);

        let lock_target_numerator = numerator.mul(network_circulating_supply);
        let lock_target_denominator = denominator;

        let power_share_numerator: BigInt = size.0.into();
        let power_share_denominator = BigInt::max(
            BigInt::max(raw_byte_power, baseline_power),
            power_share_numerator.clone(),
        );

        let collateral_numerator = lock_target_numerator.mul(power_share_numerator);
        let collateral_denominator = lock_target_denominator.mul(power_share_denominator);

        let min_collateral = collateral_numerator
            .atto()
            .div_euclid(&collateral_denominator);

        (
            TokenAmount::from_atto(min_collateral),
            TOTAL_FILECOIN.clone(),
        )
    }
}
