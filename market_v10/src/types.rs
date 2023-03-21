// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime_v10::Array;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::bigint::{bigint_ser, BigInt};
use super::deal::{DealProposal, DealState};

pub const PROPOSALS_AMT_BITWIDTH: u32 = 5;
pub const STATES_AMT_BITWIDTH: u32 = 6;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Eq, PartialEq)]
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
