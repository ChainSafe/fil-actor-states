// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::state::State;
pub use self::types::*;

mod state;
mod types;

/// Static method numbers for builtin-actor private dispatch.
/// The methods are also expected to be exposed via FRC-XXXX standard calling convention,
/// with numbers determined by name.
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    // Non-standard.
    Mint = 2,
    Destroy = 3,
    // Static method numbers for token standard methods, for private use.
    Name = 10,
    Symbol = 11,
    TotalSupply = 12,
    BalanceOf = 13,
    Transfer = 14,
    TransferFrom = 15,
    IncreaseAllowance = 16,
    DecreaseAllowance = 17,
    RevokeAllowance = 18,
    Burn = 19,
    BurnFrom = 20,
    Allowance = 21,
}
