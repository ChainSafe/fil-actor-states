// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::deal::*;
pub use self::state::*;
// pub use self::types::*;

pub mod balance_table;
mod deal;
mod state;
mod types;

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
}
