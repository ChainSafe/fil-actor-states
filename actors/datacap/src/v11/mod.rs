// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared3::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::state::State;
pub use self::types::*;

mod state;
mod types;

/// Datacap actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    // Deprecated in v10
    // Mint = 2,
    // Destroy = 3,
    // Name = 10,
    // Symbol = 11,
    // TotalSupply = 12,
    // BalanceOf = 13,
    // Transfer = 14,
    // TransferFrom = 15,
    // IncreaseAllowance = 16,
    // DecreaseAllowance = 17,
    // RevokeAllowance = 18,
    // Burn = 19,
    // BurnFrom = 20,
    // Allowance = 21,
    // Method numbers derived from FRC-0042 standards
    MintExported = frc42_macros::method_hash!("Mint"),
    DestroyExported = frc42_macros::method_hash!("Destroy"),
    NameExported = frc42_macros::method_hash!("Name"),
    SymbolExported = frc42_macros::method_hash!("Symbol"),
    GranularityExported = frc42_macros::method_hash!("Granularity"),
    TotalSupplyExported = frc42_macros::method_hash!("TotalSupply"),
    BalanceExported = frc42_macros::method_hash!("Balance"),
    TransferExported = frc42_macros::method_hash!("Transfer"),
    TransferFromExported = frc42_macros::method_hash!("TransferFrom"),
    IncreaseAllowanceExported = frc42_macros::method_hash!("IncreaseAllowance"),
    DecreaseAllowanceExported = frc42_macros::method_hash!("DecreaseAllowance"),
    RevokeAllowanceExported = frc42_macros::method_hash!("RevokeAllowance"),
    BurnExported = frc42_macros::method_hash!("Burn"),
    BurnFromExported = frc42_macros::method_hash!("BurnFrom"),
    AllowanceExported = frc42_macros::method_hash!("Allowance"),
}
