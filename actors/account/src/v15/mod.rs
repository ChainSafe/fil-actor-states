// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::state::State;
pub use types::*;

mod state;
pub mod types;

/// Account actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PubkeyAddress = 2,
    // Deprecated in v10
    // AuthenticateMessage = 3,
    AuthenticateMessageExported = frc42_macros::method_hash!("AuthenticateMessage"),
}
