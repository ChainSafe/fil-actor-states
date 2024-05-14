// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{
    from_padded_piece_size_v2_to_v3, from_padded_piece_size_v2_to_v4, from_policy_v13_to_v11,
    from_policy_v13_to_v12, from_token_v2_to_v3, from_token_v2_to_v4, from_token_v3_to_v2,
    from_token_v4_to_v2,
};
use crate::io::get_obj;
use anyhow::Context;
use cid::Cid;
use fil_actor_market_state::v11::policy::deal_provider_collateral_bounds as deal_provider_collateral_bounds_v11;
use fil_actor_market_state::v12::policy::deal_provider_collateral_bounds as deal_provider_collateral_bounds_v12;
use fil_actor_market_state::v13::policy::deal_provider_collateral_bounds as deal_provider_collateral_bounds_v13;
use fil_actor_miner_state::v11::initial_pledge_for_power as initial_pledge_for_power_v11;
use fil_actor_miner_state::v12::initial_pledge_for_power as initial_pledge_for_power_v12;
use fil_actor_miner_state::v13::initial_pledge_for_power as initial_pledge_for_power_v13;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::bigint::Integer;
use fvm_shared::sector::StoragePower;
use fvm_shared::smooth::FilterEstimate;
use fvm_shared::{address::Address, econ::TokenAmount, piece::PaddedPieceSize, TOTAL_FILECOIN};
use num::BigInt;
use serde::Serialize;
use std::cmp::max;

use crate::Policy;

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
            State::V10(st) => from_token_v3_to_v2(&st.into_total_storage_power_reward()),
            State::V11(st) => from_token_v3_to_v2(&st.into_total_storage_power_reward()),
            State::V12(st) => from_token_v4_to_v2(&st.into_total_storage_power_reward()),
            State::V13(st) => from_token_v4_to_v2(&st.into_total_storage_power_reward()),
        }
    }

    /// The baseline power the network is targeting at this state's epoch.
    pub fn this_epoch_baseline_power(&self) -> &StoragePower {
        match self {
            State::V8(st) => &st.this_epoch_baseline_power,
            State::V9(st) => &st.this_epoch_baseline_power,
            State::V10(st) => &st.this_epoch_baseline_power,
            State::V11(st) => &st.this_epoch_baseline_power,
            State::V12(st) => &st.this_epoch_baseline_power,
            State::V13(st) => &st.this_epoch_baseline_power,
        }
    }

    pub fn pre_commit_deposit_for_power(
        &self,
        network_qa_power: FilterEstimate,
        sector_weight: StoragePower,
    ) -> anyhow::Result<TokenAmount> {
        match self {
            State::V8(_st) => anyhow::bail!("unimplemented"),
            State::V9(_st) => anyhow::bail!("unimplemented"),
            State::V10(_st) => anyhow::bail!("unimplemented"),
            State::V11(st) => Ok(from_token_v3_to_v2(&st.pre_commit_deposit_for_power(
                &st.this_epoch_reward_smoothed,
                &fvm_shared3::smooth::FilterEstimate {
                    position: network_qa_power.position,
                    velocity: network_qa_power.velocity,
                },
                &sector_weight,
            ))),
            State::V12(st) => Ok(from_token_v4_to_v2(&st.pre_commit_deposit_for_power(
                &st.this_epoch_reward_smoothed,
                &fvm_shared4::smooth::FilterEstimate {
                    position: network_qa_power.position,
                    velocity: network_qa_power.velocity,
                },
                &sector_weight,
            ))),
            State::V13(st) => Ok(from_token_v4_to_v2(&st.pre_commit_deposit_for_power(
                &st.this_epoch_reward_smoothed,
                &fvm_shared4::smooth::FilterEstimate {
                    position: network_qa_power.position,
                    velocity: network_qa_power.velocity,
                },
                &sector_weight,
            ))),
        }
    }

    // The code for versions lower than `v11` does not exist in the original Rust repo, but it does
    // exist for Lotus. The logic is exactly the same for all the versions, therefore it has been
    // decided to introduce a shared helper for all of these versions to match Lotus behaviour.
    fn deal_provider_collateral_bounds_pre_v11(
        &self,
        policy: &Policy,
        size: PaddedPieceSize,
        network_raw_power: &StoragePower,
        baseline_power: &StoragePower,
        network_circulating_supply: &TokenAmount,
    ) -> (TokenAmount, TokenAmount) {
        // minimumProviderCollateral = ProviderCollateralSupplyTarget * normalizedCirculatingSupply
        // normalizedCirculatingSupply = networkCirculatingSupply * dealPowerShare
        // dealPowerShare = dealRawPower / max(BaselinePower(t), NetworkRawPower(t), dealRawPower)

        let lock_target_num =
            network_circulating_supply * policy.prov_collateral_percent_supply_num;
        let power_share_num = BigInt::from(size.0);
        let power_share_denom =
            max(max(network_raw_power, baseline_power), &power_share_num).clone();

        let num: BigInt = power_share_num * lock_target_num.atto();
        let denom: BigInt = power_share_denom * policy.prov_collateral_percent_supply_denom;
        (
            TokenAmount::from_atto(num.div_floor(&denom)),
            TOTAL_FILECOIN.clone(),
        )
    }

    pub fn deal_provider_collateral_bounds(
        &self,
        policy: &Policy,
        size: PaddedPieceSize,
        raw_byte_power: &StoragePower,
        baseline_power: &StoragePower,
        network_circulating_supply: &TokenAmount,
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
                    &from_policy_v13_to_v11(policy),
                    from_padded_piece_size_v2_to_v3(size),
                    raw_byte_power,
                    baseline_power,
                    &from_token_v2_to_v3(network_circulating_supply),
                );
                (from_token_v3_to_v2(&min), from_token_v3_to_v2(&max))
            }
            State::V12(_) => {
                let (min, max) = deal_provider_collateral_bounds_v12(
                    &from_policy_v13_to_v12(policy),
                    from_padded_piece_size_v2_to_v4(size),
                    raw_byte_power,
                    baseline_power,
                    &from_token_v2_to_v4(network_circulating_supply),
                );
                (from_token_v4_to_v2(&min), from_token_v4_to_v2(&max))
            }
            State::V13(_) => {
                let (min, max) = deal_provider_collateral_bounds_v13(
                    policy,
                    from_padded_piece_size_v2_to_v4(size),
                    raw_byte_power,
                    baseline_power,
                    &from_token_v2_to_v4(network_circulating_supply),
                );
                (from_token_v4_to_v2(&min), from_token_v4_to_v2(&max))
            }
        }
    }

    pub fn initial_pledge_for_power(
        &self,
        qa_power: &StoragePower,
        _network_total_pledge: TokenAmount,
        network_qa_power: FilterEstimate,
        circ_supply: &TokenAmount,
    ) -> anyhow::Result<TokenAmount> {
        match self {
            State::V8(_st) => anyhow::bail!("unimplemented"),
            State::V9(_st) => anyhow::bail!("unimplemented"),
            State::V10(_st) => anyhow::bail!("unimplemented"),
            State::V11(st) => {
                let pledge = initial_pledge_for_power_v11(
                    qa_power,
                    &st.this_epoch_baseline_power,
                    &st.this_epoch_reward_smoothed,
                    &fvm_shared3::smooth::FilterEstimate {
                        position: network_qa_power.position,
                        velocity: network_qa_power.velocity,
                    },
                    &from_token_v2_to_v3(circ_supply),
                );
                Ok(from_token_v3_to_v2(&pledge))
            }
            State::V12(st) => {
                let pledge = initial_pledge_for_power_v12(
                    qa_power,
                    &st.this_epoch_baseline_power,
                    &st.this_epoch_reward_smoothed,
                    &fvm_shared4::smooth::FilterEstimate {
                        position: network_qa_power.position,
                        velocity: network_qa_power.velocity,
                    },
                    &from_token_v2_to_v4(circ_supply),
                );
                Ok(from_token_v4_to_v2(&pledge))
            }
            State::V13(st) => {
                let pledge = initial_pledge_for_power_v13(
                    qa_power,
                    &st.this_epoch_baseline_power,
                    &st.this_epoch_reward_smoothed,
                    &fvm_shared4::smooth::FilterEstimate {
                        position: network_qa_power.position,
                        velocity: network_qa_power.velocity,
                    },
                    &from_token_v2_to_v4(circ_supply),
                );
                Ok(from_token_v4_to_v2(&pledge))
            }
        }
    }
}
