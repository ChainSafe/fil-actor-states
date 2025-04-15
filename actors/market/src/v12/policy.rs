// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::v12::runtime::Policy;
use fvm_shared3::TOTAL_FILECOIN;
use fvm_shared4::bigint::{BigInt, Integer};
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::piece::PaddedPieceSize;
use fvm_shared4::sector::StoragePower;
use std::cmp::max;

pub mod detail {
    /// Maximum length of a deal label.
    pub const DEAL_MAX_LABEL_SIZE: usize = 256;
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
        TokenAmount::from_atto(TOTAL_FILECOIN.atto().clone()),
    )
}

/// Penalty to provider deal collateral if the deadline expires before sector commitment.
pub(super) fn collateral_penalty_for_deal_activation_missed(
    provider_collateral: TokenAmount,
) -> TokenAmount {
    provider_collateral
}
