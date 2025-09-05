// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::BTreeSet;

use fvm_actor_utils::receiver::UniversalReceiverParams;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared4::METHOD_CONSTRUCTOR;
use fvm_shared4::MethodNum;
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;
use num_derive::FromPrimitive;
use num_traits::Zero;

use fil_actors_runtime::FIRST_EXPORTED_METHOD_NUMBER;
use fil_actors_runtime::cbor::serialize_vec;
use fil_actors_runtime::runtime::{ActorCode, Primitives, Runtime};
use fil_actors_runtime::{
    ActorContext, ActorError, AsActorError, INIT_ACTOR_ADDR, actor_dispatch, actor_error,
    extract_send_result, resolve_to_actor_id,
};

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
