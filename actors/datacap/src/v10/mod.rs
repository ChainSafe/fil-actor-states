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
    MintExported = frc42_dispatch::method_hash!("Mint"),
    DestroyExported = frc42_dispatch::method_hash!("Destroy"),
    NameExported = frc42_dispatch::method_hash!("Name"),
    SymbolExported = frc42_dispatch::method_hash!("Symbol"),
    GranularityExported = frc42_dispatch::method_hash!("Granularity"),
    TotalSupplyExported = frc42_dispatch::method_hash!("TotalSupply"),
    BalanceExported = frc42_dispatch::method_hash!("Balance"),
    TransferExported = frc42_dispatch::method_hash!("Transfer"),
    TransferFromExported = frc42_dispatch::method_hash!("TransferFrom"),
    IncreaseAllowanceExported = frc42_dispatch::method_hash!("IncreaseAllowance"),
    DecreaseAllowanceExported = frc42_dispatch::method_hash!("DecreaseAllowance"),
    RevokeAllowanceExported = frc42_dispatch::method_hash!("RevokeAllowance"),
    BurnExported = frc42_dispatch::method_hash!("Burn"),
    BurnFromExported = frc42_dispatch::method_hash!("BurnFrom"),
    AllowanceExported = frc42_dispatch::method_hash!("Allowance"),
}
