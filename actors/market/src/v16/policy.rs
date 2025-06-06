// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::cmp::max;

use fil_actors_shared::v16::network::EPOCHS_IN_DAY;
use fil_actors_shared::v16::runtime::Policy;
use fvm_shared4::bigint::{BigInt, Integer};
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::piece::PaddedPieceSize;
use fvm_shared4::sector::StoragePower;
use lazy_static::lazy_static;
use num_traits::Zero;

pub mod detail {
    /// Maximum length of a deal label.
    pub const DEAL_MAX_LABEL_SIZE: usize = 256;
}

lazy_static! {
    /// Total (assumed) Filecoin available to the network. This is only used to bound the maximum
    /// deal collateral and price.
    pub static ref TOTAL_FILECOIN: TokenAmount = TokenAmount::from_whole(2_000_000_000);
}

/// Bounds (inclusive) on deal duration.
pub(super) fn _deal_duration_bounds(_size: PaddedPieceSize) -> (ChainEpoch, ChainEpoch) {
    (180 * EPOCHS_IN_DAY, 1278 * EPOCHS_IN_DAY)
}

pub(super) fn _deal_price_per_epoch_bounds(
    _size: PaddedPieceSize,
    _duration: ChainEpoch,
) -> (TokenAmount, &'static TokenAmount) {
    (TokenAmount::zero(), &TOTAL_FILECOIN)
}

pub fn deal_provider_collateral_bounds(
    policy: &Policy,
    size: PaddedPieceSize,
    network_raw_power: &StoragePower,
    baseline_power: &StoragePower,
    network_circulating_supply: &TokenAmount,
) -> (TokenAmount, TokenAmount) {
    // minimumProviderCollateral = ProviderCollateralSupplyTarget * normalizedCirculatingSupply
    // normalizedCirculatingSupply = networkCirculatingSupply * dealPowerShare
    // dealPowerShare = dealRawPower / max(BaselinePower(t), NetworkRawPower(t), dealRawPower)

    let lock_target_num = network_circulating_supply * policy.prov_collateral_percent_supply_num;
    let power_share_num = BigInt::from(size.0);
    let power_share_denom = max(max(network_raw_power, baseline_power), &power_share_num).clone();

    let num: BigInt = power_share_num * lock_target_num.atto();
    let denom: BigInt = power_share_denom * policy.prov_collateral_percent_supply_denom;
    (
        TokenAmount::from_atto(num.div_floor(&denom)),
        TOTAL_FILECOIN.clone(),
    )
}

pub(super) fn _deal_client_collateral_bounds(
    _: PaddedPieceSize,
    _: ChainEpoch,
) -> (TokenAmount, TokenAmount) {
    (TokenAmount::zero(), TOTAL_FILECOIN.clone()) // PARAM_FINISH
}

/// Penalty to provider deal collateral if the deadline expires before sector commitment.
pub(super) fn collateral_penalty_for_deal_activation_missed(
    provider_collateral: TokenAmount,
) -> TokenAmount {
    provider_collateral
}
