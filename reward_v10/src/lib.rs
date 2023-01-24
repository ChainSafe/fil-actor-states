// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime_v10::runtime::{ActorCode, Runtime};
use fil_actors_runtime_v10::{
    actor_dispatch, actor_error, ActorError, BURNT_FUNDS_ACTOR_ADDR, EXPECTED_LEADERS_PER_EPOCH,
    STORAGE_POWER_ACTOR_ADDR, SYSTEM_ACTOR_ADDR,
};

use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::bigint::bigint_ser::BigIntDe;
use fvm_shared::econ::TokenAmount;
use fvm_shared::{METHOD_CONSTRUCTOR, METHOD_SEND};
use log::{error, warn};
use num_derive::FromPrimitive;

pub use self::logic::*;
pub use self::state::{Reward, State, VestingFunction};
pub use self::types::*;

#[cfg(feature = "fil-actor")]
fil_actors_runtime_v10::wasm_trampoline!(Actor);

pub(crate) mod expneg;
mod logic;
mod state;
pub mod testing;
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
}
