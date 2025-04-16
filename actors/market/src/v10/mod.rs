// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::v10::FIRST_ACTOR_SPECIFIC_EXIT_CODE;
use fvm_shared3::METHOD_CONSTRUCTOR;
use fvm_shared3::error::ExitCode;
use num_derive::FromPrimitive;

pub use self::deal::*;
pub use self::state::*;
pub use self::types::*;

pub mod balance_table;
pub mod policy;

mod deal;
mod state;
mod types;

pub const NO_ALLOCATION_ID: u64 = 0;

// An exit code indicating that information about a past deal is no longer available.
pub const EX_DEAL_EXPIRED: ExitCode = ExitCode::new(FIRST_ACTOR_SPECIFIC_EXIT_CODE);

/// Market actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    AddBalance = 2,
    WithdrawBalance = 3,
    PublishStorageDeals = 4,
    VerifyDealsForActivation = 5,
    ActivateDeals = 6,
    OnMinerSectorsTerminate = 7,
    ComputeDataCommitment = 8,
    CronTick = 9,
    // Method numbers derived from FRC-0042 standards
    AddBalanceExported = frc42_macros::method_hash!("AddBalance"),
    WithdrawBalanceExported = frc42_macros::method_hash!("WithdrawBalance"),
    PublishStorageDealsExported = frc42_macros::method_hash!("PublishStorageDeals"),
    GetBalanceExported = frc42_macros::method_hash!("GetBalance"),
    GetDealDataCommitmentExported = frc42_macros::method_hash!("GetDealDataCommitment"),
    GetDealClientExported = frc42_macros::method_hash!("GetDealClient"),
    GetDealProviderExported = frc42_macros::method_hash!("GetDealProvider"),
    GetDealLabelExported = frc42_macros::method_hash!("GetDealLabel"),
    GetDealTermExported = frc42_macros::method_hash!("GetDealTerm"),
    GetDealTotalPriceExported = frc42_macros::method_hash!("GetDealTotalPrice"),
    GetDealClientCollateralExported = frc42_macros::method_hash!("GetDealClientCollateral"),
    GetDealProviderCollateralExported = frc42_macros::method_hash!("GetDealProviderCollateral"),
    GetDealVerifiedExported = frc42_macros::method_hash!("GetDealVerified"),
    GetDealActivationExported = frc42_macros::method_hash!("GetDealActivation"),
}
