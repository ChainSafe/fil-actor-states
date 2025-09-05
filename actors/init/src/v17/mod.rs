// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Runtime};

use fil_actors_runtime::{
    ActorContext, ActorError, AsActorError, EAM_ACTOR_ADDR, SYSTEM_ACTOR_ADDR, actor_dispatch,
    actor_error, extract_send_result,
};
use fvm_shared4::address::Address;
use fvm_shared4::error::ExitCode;
use fvm_shared4::{ActorID, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

pub use self::state::State;
pub use self::types::*;

mod state;
mod types;

/// Init actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Exec = 2,
    Exec4 = 3,
}
