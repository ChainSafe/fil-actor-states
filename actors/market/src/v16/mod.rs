// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::v16::FIRST_ACTOR_SPECIFIC_EXIT_CODE;
use fvm_shared4::error::ExitCode;
use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::deal::*;
pub use self::state::*;
pub use self::types::*;

// exports for testing
pub mod balance_table;
#[doc(hidden)]
pub mod ext;
pub mod policy;

mod deal;
mod state;
mod types;

pub const NO_ALLOCATION_ID: u64 = 0;

// Indicates that information about a past deal is no longer available.
pub const EX_DEAL_EXPIRED: ExitCode = ExitCode::new(FIRST_ACTOR_SPECIFIC_EXIT_CODE);
// Indicates that information about a deal's activation is not yet available.
pub const EX_DEAL_NOT_ACTIVATED: ExitCode = ExitCode::new(FIRST_ACTOR_SPECIFIC_EXIT_CODE + 1);

/// Market actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    AddBalance = 2,
    WithdrawBalance = 3,
    PublishStorageDeals = 4,
    VerifyDealsForActivation = 5,
    BatchActivateDeals = 6,
    OnMinerSectorsTerminate = 7,
    // ComputeDataCommitment = 8, // Deprecated
    CronTick = 9,
    // Method numbers derived from FRC-0042 standards
    AddBalanceExported = frc42_dispatch::method_hash!("AddBalance"),
    WithdrawBalanceExported = frc42_dispatch::method_hash!("WithdrawBalance"),
    PublishStorageDealsExported = frc42_dispatch::method_hash!("PublishStorageDeals"),
    GetBalanceExported = frc42_dispatch::method_hash!("GetBalance"),
    GetDealDataCommitmentExported = frc42_dispatch::method_hash!("GetDealDataCommitment"),
    GetDealClientExported = frc42_dispatch::method_hash!("GetDealClient"),
    GetDealProviderExported = frc42_dispatch::method_hash!("GetDealProvider"),
    GetDealLabelExported = frc42_dispatch::method_hash!("GetDealLabel"),
    GetDealTermExported = frc42_dispatch::method_hash!("GetDealTerm"),
    GetDealTotalPriceExported = frc42_dispatch::method_hash!("GetDealTotalPrice"),
    GetDealClientCollateralExported = frc42_dispatch::method_hash!("GetDealClientCollateral"),
    GetDealProviderCollateralExported = frc42_dispatch::method_hash!("GetDealProviderCollateral"),
    GetDealVerifiedExported = frc42_dispatch::method_hash!("GetDealVerified"),
    GetDealActivationExported = frc42_dispatch::method_hash!("GetDealActivation"),
    GetDealSectorExported = frc42_dispatch::method_hash!("GetDealSector"),
    SettleDealPaymentsExported = frc42_dispatch::method_hash!("SettleDealPayments"),
    SectorContentChangedExported = ext::miner::SECTOR_CONTENT_CHANGED,
}
