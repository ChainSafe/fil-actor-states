// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{
    from_padded_piece_size_v2_to_v3, from_padded_piece_size_v2_to_v4, from_policy_v10_to_v11,
    from_policy_v10_to_v12, from_policy_v10_to_v13, from_token_v2_to_v3, from_token_v2_to_v4,
    from_token_v3_to_v2, from_token_v4_to_v2,
};
use crate::io::get_obj;
use anyhow::Context;
use cid::Cid;
use fil_actor_market_state::v11::policy::deal_provider_collateral_bounds as deal_provider_collateral_bounds_v11;
use fil_actor_market_state::v12::policy::deal_provider_collateral_bounds as deal_provider_collateral_bounds_v12;
use fil_actor_market_state::v13::policy::deal_provider_collateral_bounds as deal_provider_collateral_bounds_v13;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::sector::StoragePower;
use fvm_shared::{address::Address, econ::TokenAmount, piece::PaddedPieceSize, TOTAL_FILECOIN};
use num::traits::Euclid;
use num::BigInt;
use serde::Serialize;
use std::convert::Into;
use std::ops::Mul;

use fil_actors_shared::v10::runtime::Policy;

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

    fn deal_provider_collateral_bounds_pre_v11(
        &self,
        policy: &Policy,
        size: PaddedPieceSize,
        raw_byte_power: StoragePower,
        baseline_power: StoragePower,
        network_circulating_supply: TokenAmount,
    ) -> (TokenAmount, TokenAmount) {
        // The percentage of normalized circulating supply that must be covered by provider
        // collateral in a deal.
        // See: https://github.com/filecoin-project/go-state-types/blob/master/builtin/v12/market/policy.go#L9-L14.
        let numerator = BigInt::from(policy.prov_collateral_percent_supply_num);
        let denominator = BigInt::from(policy.prov_collateral_percent_supply_denom);

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

    pub fn deal_provider_collateral_bounds(
        &self,
        policy: &Policy,
        size: PaddedPieceSize,
        raw_byte_power: StoragePower,
        baseline_power: StoragePower,
        network_circulating_supply: TokenAmount,
    ) -> (TokenAmount, TokenAmount) {
        match self {
            State::V8(_) => self.deal_provider_collateral_bounds_pre_v11(
                policy,
                size,
                raw_byte_power,
                baseline_power,
                network_circulating_supply,
            ),
            State::V9(_) => self.deal_provider_collateral_bounds_pre_v11(
                policy,
                size,
                raw_byte_power,
                baseline_power,
                network_circulating_supply,
            ),
            State::V10(_) => self.deal_provider_collateral_bounds_pre_v11(
                policy,
                size,
                raw_byte_power,
                baseline_power,
                network_circulating_supply,
            ),
            State::V11(_) => {
                let (min, max) = deal_provider_collateral_bounds_v11(
                    &from_policy_v10_to_v11(policy),
                    from_padded_piece_size_v2_to_v3(size),
                    &raw_byte_power,
                    &baseline_power,
                    &from_token_v2_to_v3(network_circulating_supply),
                );
                (from_token_v3_to_v2(min), from_token_v3_to_v2(max))
            }
            State::V12(_) => {
                let (min, max) = deal_provider_collateral_bounds_v12(
                    &from_policy_v10_to_v12(policy),
                    from_padded_piece_size_v2_to_v4(size),
                    &raw_byte_power,
                    &baseline_power,
                    &from_token_v2_to_v4(network_circulating_supply),
                );
                (from_token_v4_to_v2(min), from_token_v4_to_v2(max))
            }
            State::V13(_) => {
                let (min, max) = deal_provider_collateral_bounds_v13(
                    &from_policy_v10_to_v13(policy),
                    from_padded_piece_size_v2_to_v4(size),
                    &raw_byte_power,
                    &baseline_power,
                    &from_token_v2_to_v4(network_circulating_supply),
                );
                (from_token_v4_to_v2(min), from_token_v4_to_v2(max))
            }
        }
    }
}
