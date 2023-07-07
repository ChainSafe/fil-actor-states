// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared3::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use self::state::Allocation;
pub use self::state::Claim;
pub use self::state::State;
pub use types::*;

pub mod ext;
pub mod state;
mod types;

/// Account actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    AddVerifier = 2,
    RemoveVerifier = 3,
    AddVerifiedClient = 4,
    // UseBytes = 5,     // Deprecated
    // RestoreBytes = 6, // Deprecated
    RemoveVerifiedClientDataCap = 7,
    RemoveExpiredAllocations = 8,
    ClaimAllocations = 9,
    GetClaims = 10,
    ExtendClaimTerms = 11,
    RemoveExpiredClaims = 12,
    // Method numbers derived from FRC-0042 standards
    AddVerifiedClientExported = frc42_macros::method_hash!("AddVerifiedClient"),
    RemoveExpiredAllocationsExported = frc42_macros::method_hash!("RemoveExpiredAllocations"),
    GetClaimsExported = frc42_macros::method_hash!("GetClaims"),
    ExtendClaimTermsExported = frc42_macros::method_hash!("ExtendClaimTerms"),
    RemoveExpiredClaimsExported = frc42_macros::method_hash!("RemoveExpiredClaims"),
    UniversalReceiverHook = frc42_macros::method_hash!("Receive"),
}
