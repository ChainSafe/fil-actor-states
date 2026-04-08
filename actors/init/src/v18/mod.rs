// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::state::*;
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
