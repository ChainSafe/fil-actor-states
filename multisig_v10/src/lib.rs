// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

use fil_actors_runtime_v10::cbor::serialize_vec;
use fil_actors_runtime_v10::make_map_with_root;
use fil_actors_runtime_v10::runtime::Primitives;

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
    UniversalReceiverHook = frc42_dispatch::method_hash!("Receive"),
}
