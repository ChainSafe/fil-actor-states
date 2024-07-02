// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared4::error::ExitCode;
use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::policy::*;
pub use self::state::*;
pub use self::types::*;

#[doc(hidden)]
pub mod ext;
mod policy;
mod state;
mod types;

// * Updated to specs-actors commit: 999e57a151cc7ada020ca2844b651499ab8c0dec (v3.0.1)

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
    // OnConsensusFault = 7, // Deprecated v2
    // SubmitPoRepForBulkVerify = 8, // Deprecated
    CurrentTotalPower = 9,
    // Method numbers derived from FRC-0042 standards
    CreateMinerExported = frc42_dispatch::method_hash!("CreateMiner"),
    NetworkRawPowerExported = frc42_dispatch::method_hash!("NetworkRawPower"),
    MinerRawPowerExported = frc42_dispatch::method_hash!("MinerRawPower"),
    MinerCountExported = frc42_dispatch::method_hash!("MinerCount"),
    MinerConsensusCountExported = frc42_dispatch::method_hash!("MinerConsensusCount"),
}

pub const ERR_TOO_MANY_PROVE_COMMITS: ExitCode = ExitCode::new(32);
