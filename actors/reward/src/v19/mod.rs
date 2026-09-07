// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::logic::*;
pub use self::state::{
    DENOM, ExplicitDistribution, MAX_PAYABLE_ROWS_PER_STREAM, MAX_PENDING_WRITES, MAX_RECIPIENTS,
    MAX_STREAMS, MAX_TOMBSTONE_ROWS, PendingWrite, PendingWriteOp, RecipientAmount, RecipientShare,
    RecipientTable, State, Stream, StreamAccrual, StreamId, StreamsState, Tombstone, WeightRecord,
};
pub use self::streams::*;
pub use self::types::*;

pub(crate) mod expneg;
mod logic;
mod state;
mod streams;
mod types;

// only exported for tests
#[doc(hidden)]
pub mod ext;

// * Updated to specs-actors commit: 999e57a151cc7ada020ca2844b651499ab8c0dec (v3.0.1)

/// PenaltyMultiplier is the factor miner penalties are scaled up by
pub const PENALTY_MULTIPLIER: u64 = 3;

/// Reward actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    AwardBlockReward = 2,
    ThisEpochReward = 3,
    UpdateNetworkKPI = 4,
    // Method numbers derived from FRC-0042 standards
    SetWeightRecordsExported = frc42_dispatch::method_hash!("SetWeightRecords"),
    StepWeightRecordsExported = frc42_dispatch::method_hash!("StepWeightRecords"),
    RegisterStreamExported = frc42_dispatch::method_hash!("RegisterStream"),
    RemoveStreamExported = frc42_dispatch::method_hash!("RemoveStream"),
    SetDistributionExported = frc42_dispatch::method_hash!("SetDistribution"),
    CancelPendingExported = frc42_dispatch::method_hash!("CancelPending"),
    SetSharesExported = frc42_dispatch::method_hash!("SetShares"),
    ClaimExported = frc42_dispatch::method_hash!("Claim"),
}
