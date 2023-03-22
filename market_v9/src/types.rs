// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::deal::{DealProposal, DealState};
use fil_actors_runtime_v9::Array;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::bigint::{bigint_ser, BigInt};

pub const PROPOSALS_AMT_BITWIDTH: u32 = 5;
pub const STATES_AMT_BITWIDTH: u32 = 6;

pub type AllocationID = u64;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Default)]
pub struct DealSpaces {
    #[serde(with = "bigint_ser")]
    pub deal_space: BigInt,
    #[serde(with = "bigint_ser")]
    pub verified_deal_space: BigInt,
}

/// A specialization of a array to deals.
pub type DealArray<'bs, BS> = Array<'bs, DealProposal, BS>;

/// A specialization of a array to deals.
pub type DealMetaArray<'bs, BS> = Array<'bs, DealState, BS>;
