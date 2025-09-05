// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use cid::Cid;
use cid::multihash::Multihash;
use fil_actors_runtime::reward::ThisEpochRewardReturn;
use frc46_token::token::types::{BalanceReturn, TransferFromParams, TransferFromReturn};
use fvm_ipld_bitfield::BitField;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_encoding::{DAG_CBOR, RawBytes};
use fvm_ipld_hamt::BytesKey;
use fvm_shared4::address::Address;
use fvm_shared4::bigint::BigInt;
use fvm_shared4::clock::{ChainEpoch, EPOCH_UNDEFINED};
use fvm_shared4::crypto::hash::SupportedHashes;
use fvm_shared4::deal::DealID;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;
use fvm_shared4::piece::PieceInfo;
use fvm_shared4::sector::{RegisteredSealProof, SectorNumber, SectorSize, StoragePower};
use fvm_shared4::sys::SendFlags;
use fvm_shared4::{ActorID, METHOD_CONSTRUCTOR, METHOD_SEND};
use integer_encoding::VarInt;
use log::{info, warn};
use num_derive::FromPrimitive;
use num_traits::Zero;

use fil_actors_runtime::cbor::{deserialize, serialize};
use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Policy, Runtime};
use fil_actors_runtime::{
    ActorContext, ActorDowncast, ActorError, AsActorError, BURNT_FUNDS_ACTOR_ADDR, CRON_ACTOR_ADDR,
    DATACAP_TOKEN_ACTOR_ADDR, REWARD_ACTOR_ADDR, STORAGE_POWER_ACTOR_ADDR, SYSTEM_ACTOR_ADDR,
    VERIFIED_REGISTRY_ACTOR_ADDR, actor_dispatch, actor_error, deserialize_block,
};
use fil_actors_runtime::{BatchReturnGen, FIRST_ACTOR_SPECIFIC_EXIT_CODE, extract_send_result};

use crate::balance_table::BalanceTable;
use crate::ext::verifreg::{AllocationID, AllocationRequest};

pub use self::deal::*;
use self::policy::*;
pub use self::state::*;
pub use self::types::*;

// exports for testing
pub mod balance_table;
#[doc(hidden)]
pub mod ext;
pub mod policy;

mod deal;
mod state;
mod types;

pub const NO_ALLOCATION_ID: u64 = 0;

// Indicates that information about a past deal is no longer available.
pub const EX_DEAL_EXPIRED: ExitCode = ExitCode::new(FIRST_ACTOR_SPECIFIC_EXIT_CODE);
// Indicates that information about a deal's activation is not yet available.
pub const EX_DEAL_NOT_ACTIVATED: ExitCode = ExitCode::new(FIRST_ACTOR_SPECIFIC_EXIT_CODE + 1);

/// Market actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    AddBalance = 2,
    WithdrawBalance = 3,
    PublishStorageDeals = 4,
    VerifyDealsForActivation = 5,
    BatchActivateDeals = 6,
    OnMinerSectorsTerminate = 7,
    // ComputeDataCommitment = 8, // Deprecated
    CronTick = 9,
    // Method numbers derived from FRC-0042 standards
    AddBalanceExported = frc42_dispatch::method_hash!("AddBalance"),
    WithdrawBalanceExported = frc42_dispatch::method_hash!("WithdrawBalance"),
    PublishStorageDealsExported = frc42_dispatch::method_hash!("PublishStorageDeals"),
    GetBalanceExported = frc42_dispatch::method_hash!("GetBalance"),
    GetDealDataCommitmentExported = frc42_dispatch::method_hash!("GetDealDataCommitment"),
    GetDealClientExported = frc42_dispatch::method_hash!("GetDealClient"),
    GetDealProviderExported = frc42_dispatch::method_hash!("GetDealProvider"),
    GetDealLabelExported = frc42_dispatch::method_hash!("GetDealLabel"),
    GetDealTermExported = frc42_dispatch::method_hash!("GetDealTerm"),
    GetDealTotalPriceExported = frc42_dispatch::method_hash!("GetDealTotalPrice"),
    GetDealClientCollateralExported = frc42_dispatch::method_hash!("GetDealClientCollateral"),
    GetDealProviderCollateralExported = frc42_dispatch::method_hash!("GetDealProviderCollateral"),
    GetDealVerifiedExported = frc42_dispatch::method_hash!("GetDealVerified"),
    GetDealActivationExported = frc42_dispatch::method_hash!("GetDealActivation"),
    GetDealSectorExported = frc42_dispatch::method_hash!("GetDealSector"),
    SettleDealPaymentsExported = frc42_dispatch::method_hash!("SettleDealPayments"),
    SectorContentChangedExported = ext::miner::SECTOR_CONTENT_CHANGED,
}
