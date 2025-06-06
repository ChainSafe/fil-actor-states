// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared4::METHOD_CONSTRUCTOR;
use fvm_shared4::error::ExitCode;
use num_derive::FromPrimitive;

pub use self::policy::*;
pub use self::state::*;
pub use self::types::*;

#[doc(hidden)]
pub mod ext;
mod policy;
mod state;
mod types;

/// GasOnSubmitVerifySeal is amount of gas charged for SubmitPoRepForBulkVerify
/// This number is empirically determined
pub mod detail {
    pub const GAS_ON_SUBMIT_VERIFY_SEAL: i64 = 34721049;
}

/// Storage power actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    /// Constructor for Storage Power Actor
    Constructor = METHOD_CONSTRUCTOR,
    CreateMiner = 2,
    UpdateClaimedPower = 3,
    EnrollCronEvent = 4,
    OnEpochTickEnd = 5,
    UpdatePledgeTotal = 6,
    // * Deprecated in v2
    // OnConsensusFault = 7,
    SubmitPoRepForBulkVerify = 8,
    CurrentTotalPower = 9,
    // Method numbers derived from FRC-0042 standards
    CreateMinerExported = frc42_macros::method_hash!("CreateMiner"),
    NetworkRawPowerExported = frc42_macros::method_hash!("NetworkRawPower"),
    MinerRawPowerExported = frc42_macros::method_hash!("MinerRawPower"),
    MinerCountExported = frc42_macros::method_hash!("MinerCount"),
    MinerConsensusCountExported = frc42_macros::method_hash!("MinerConsensusCount"),
}

pub const ERR_TOO_MANY_PROVE_COMMITS: ExitCode = ExitCode::new(32);
