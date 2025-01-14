// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub use self::state::State;
use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

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
    AuthenticateMessageExported = frc42_dispatch::method_hash!("AuthenticateMessage"),
}
