// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime_v9::network::EPOCHS_IN_DAY;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use lazy_static::lazy_static;

/// Projection period of expected sector block reward for deposit required to pre-commit a sector.
/// This deposit is lost if the pre-commitment is not timely followed up by a commitment proof.
const PRE_COMMIT_DEPOSIT_FACTOR: u64 = 20;

/// Projection period of expected sector block rewards for storage pledge required to commit a sector.
/// This pledge is lost if a sector is terminated before its full committed lifetime.
pub const INITIAL_PLEDGE_FACTOR: u64 = 20;

pub const PRE_COMMIT_DEPOSIT_PROJECTION_PERIOD: i64 =
    (PRE_COMMIT_DEPOSIT_FACTOR as ChainEpoch) * EPOCHS_IN_DAY;
pub const INITIAL_PLEDGE_PROJECTION_PERIOD: i64 =
    (INITIAL_PLEDGE_FACTOR as ChainEpoch) * EPOCHS_IN_DAY;

pub const TERMINATION_REWARD_FACTOR_NUM: u32 = 1;
pub const TERMINATION_REWARD_FACTOR_DENOM: u32 = 2;

lazy_static! {
    /// Cap on initial pledge requirement for sectors during the Space Race network.
    /// The target is 1 FIL (10**18 attoFIL) per 32 GiB.
    /// This does not divide evenly, so the result is fractionally smaller.
    static ref INITIAL_PLEDGE_MAX_PER_BYTE: TokenAmount =
        TokenAmount::from_whole(1).div_floor(32i64 << 30);

    /// Base reward for successfully disputing a window posts proofs.
    pub static ref BASE_REWARD_FOR_DISPUTED_WINDOW_POST: TokenAmount = TokenAmount::from_whole(4);

    /// Base penalty for a successful disputed window post proof.
    pub static ref BASE_PENALTY_FOR_DISPUTED_WINDOW_POST: TokenAmount = TokenAmount::from_whole(20);
}

// Projection period of expected daily sector block reward penalised when a fault is continued after initial detection.
// This guarantees that a miner pays back at least the expected block reward earned since the last successful PoSt.
// The network conservatively assumes the sector was faulty since the last time it was proven.
// This penalty is currently overly punitive for continued faults.
// FF = BR(t, ContinuedFaultProjectionPeriod)
const CONTINUED_FAULT_FACTOR_NUM: i64 = 351;
const CONTINUED_FAULT_FACTOR_DENOM: i64 = 100;
pub const CONTINUED_FAULT_PROJECTION_PERIOD: ChainEpoch =
    (EPOCHS_IN_DAY * CONTINUED_FAULT_FACTOR_NUM) / CONTINUED_FAULT_FACTOR_DENOM;

// Maximum number of lifetime days penalized when a sector is terminated.
pub const TERMINATION_LIFETIME_CAP: ChainEpoch = 140;
