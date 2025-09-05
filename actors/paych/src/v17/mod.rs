// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{
    ActorContext, ActorDowncast, ActorError, Array, actor_dispatch, actor_error, deserialize_block,
    extract_send_result, resolve_to_actor_id,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::CBOR;
use fvm_shared4::address::Address;

use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;
use fvm_shared4::sys::SendFlags;
use fvm_shared4::{METHOD_CONSTRUCTOR, METHOD_SEND};
use num_derive::FromPrimitive;
use num_traits::Zero;

pub use self::state::{LaneState, Merge, State};
pub use self::types::*;

pub mod ext;
mod state;
mod types;

// * Updated to specs-actors commit: f47f461b0588e9f0c20c999f6f129c85d669a7aa (v3.0.2)

/// Payment Channel actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    UpdateChannelState = 2,
    Settle = 3,
    Collect = 4,
}

pub const ERR_CHANNEL_STATE_UPDATE_AFTER_SETTLED: ExitCode = ExitCode::new(32);
