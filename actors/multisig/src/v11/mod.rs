// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::v11::make_map_with_root;
use fvm_shared3::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::state::*;
pub use self::types::*;

mod state;
mod types;

/// Multisig actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Propose = 2,
    Approve = 3,
    Cancel = 4,
    AddSigner = 5,
    RemoveSigner = 6,
    SwapSigner = 7,
    ChangeNumApprovalsThreshold = 8,
    LockBalance = 9,
    // Method numbers derived from FRC-0042 standards
    UniversalReceiverHook = frc42_macros::method_hash!("Receive"),
}
