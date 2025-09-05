// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use frc46_token::receiver::{FRC46_TOKEN_TYPE, FRC46TokenReceived};
use frc46_token::token::TOKEN_PRECISION;
use frc46_token::token::types::{BurnParams, TransferParams};
use fvm_actor_utils::receiver::UniversalReceiverParams;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared4::address::Address;
use fvm_shared4::bigint::BigInt;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;
use fvm_shared4::sys::SendFlags;
use fvm_shared4::{ActorID, METHOD_CONSTRUCTOR};
use log::info;
use num_derive::FromPrimitive;
use num_traits::{Signed, Zero};

use fil_actors_runtime::cbor::deserialize;
use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Policy, Runtime};
use fil_actors_runtime::{ActorContext, AsActorError, BatchReturnGen};
use fil_actors_runtime::{
    ActorError, BatchReturn, DATACAP_TOKEN_ACTOR_ADDR, STORAGE_MARKET_ACTOR_ADDR,
    SYSTEM_ACTOR_ADDR, VERIFIED_REGISTRY_ACTOR_ADDR, actor_dispatch, actor_error,
    deserialize_block, extract_send_result, resolve_to_actor_id,
};

use crate::ext::datacap::{DestroyParams, MintParams};
use crate::state::{
    DATACAP_MAP_CONFIG, DataCapMap, REMOVE_DATACAP_PROPOSALS_CONFIG, RemoveDataCapProposalMap,
};

pub use self::state::Allocation;
pub use self::state::Claim;
pub use self::state::State;
pub use self::types::*;

pub mod expiration;
pub mod ext;
pub mod state;
pub mod types;

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
    AddVerifiedClientExported = frc42_dispatch::method_hash!("AddVerifiedClient"),
    RemoveExpiredAllocationsExported = frc42_dispatch::method_hash!("RemoveExpiredAllocations"),
    GetClaimsExported = frc42_dispatch::method_hash!("GetClaims"),
    ExtendClaimTermsExported = frc42_dispatch::method_hash!("ExtendClaimTerms"),
    RemoveExpiredClaimsExported = frc42_dispatch::method_hash!("RemoveExpiredClaims"),
    UniversalReceiverHook = frc42_dispatch::method_hash!("Receive"),
}
