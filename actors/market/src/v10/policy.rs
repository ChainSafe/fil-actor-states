// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared3::econ::TokenAmount;

/// Penalty to provider deal collateral if the deadline expires before sector commitment.
pub(super) fn collateral_penalty_for_deal_activation_missed(
    provider_collateral: TokenAmount,
) -> TokenAmount {
    provider_collateral
}
