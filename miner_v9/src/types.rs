// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::Cbor;
use fvm_shared::address::Address;
use fvm_shared::bigint::bigint_ser;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::deal::DealID;
use fvm_shared::econ::TokenAmount;
use fvm_shared::sector::{RegisteredSealProof, SectorNumber};

use fil_actors_runtime_v9::DealWeight;

use crate::commd::CompactCommD;
use crate::ext::verifreg::ClaimID;

pub type CronEvent = i64;

pub const CRON_EVENT_WORKER_KEY_CHANGE: CronEvent = 0;
pub const CRON_EVENT_PROVING_DEADLINE: CronEvent = 1;
pub const CRON_EVENT_PROCESS_EARLY_TERMINATIONS: CronEvent = 2;

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct PoStPartition {
    /// Partitions are numbered per-deadline, from zero.
    pub index: u64,
    /// Sectors skipped while proving that weren't already declared faulty.
    pub skipped: BitField,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct ExpirationExtension {
    pub deadline: u64,
    pub partition: u64,
    pub sectors: BitField,
    pub new_expiration: ChainEpoch,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ExtendSectorExpiration2Params {
    pub extensions: Vec<ExpirationExtension2>,
}

impl Cbor for ExtendSectorExpiration2Params {}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct SectorClaim {
    pub sector_number: SectorNumber,
    pub maintain_claims: Vec<ClaimID>,
    pub drop_claims: Vec<ClaimID>,
}

impl Cbor for SectorClaim {}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ExpirationExtension2 {
    pub deadline: u64,
    pub partition: u64,
    pub sectors: BitField, // IDs of sectors without FIL+ claims
    pub sectors_with_claims: Vec<SectorClaim>,
    pub new_expiration: ChainEpoch,
}

impl Cbor for ExpirationExtension2 {}

// From is straightforward when there are no claim bearing sectors
impl From<&ExpirationExtension> for ExpirationExtension2 {
    fn from(e: &ExpirationExtension) -> Self {
        ExpirationExtension2 {
            deadline: e.deadline,
            partition: e.partition,
            sectors: e.sectors.clone(),
            sectors_with_claims: vec![],
            new_expiration: e.new_expiration,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct WorkerKeyChange {
    /// Must be an ID address
    pub new_worker: Address,
    pub effective_at: ChainEpoch,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct SectorPreCommitInfo {
    pub seal_proof: RegisteredSealProof,
    pub sector_number: SectorNumber,
    /// `CommR`
    pub sealed_cid: Cid,
    pub seal_rand_epoch: ChainEpoch,
    pub deal_ids: Vec<DealID>,
    pub expiration: ChainEpoch,
    /// `CommD`
    pub unsealed_cid: CompactCommD,
}

/// Information stored on-chain for a pre-committed sector.
#[derive(Debug, PartialEq, Eq, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct SectorPreCommitOnChainInfo {
    pub info: SectorPreCommitInfo,
    pub pre_commit_deposit: TokenAmount,
    pub pre_commit_epoch: ChainEpoch,
}

/// Information stored on-chain for a proven sector.
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct SectorOnChainInfo {
    pub sector_number: SectorNumber,
    /// The seal proof type implies the PoSt proofs
    pub seal_proof: RegisteredSealProof,
    /// `CommR`
    pub sealed_cid: Cid,
    pub deal_ids: Vec<DealID>,
    /// Epoch during which the sector proof was accepted
    pub activation: ChainEpoch,
    /// Epoch during which the sector expires
    pub expiration: ChainEpoch,
    /// Integral of active deals over sector lifetime
    #[serde(with = "bigint_ser")]
    pub deal_weight: DealWeight,
    /// Integral of active verified deals over sector lifetime
    #[serde(with = "bigint_ser")]
    pub verified_deal_weight: DealWeight,
    /// Pledge collected to commit this sector
    pub initial_pledge: TokenAmount,
    /// Expected one day projection of reward for sector computed at activation time
    pub expected_day_reward: TokenAmount,
    /// Expected twenty day projection of reward for sector computed at activation time
    pub expected_storage_pledge: TokenAmount,
    /// Age of sector this sector replaced or zero
    pub replaced_sector_age: ChainEpoch,
    /// Day reward of sector this sector replace or zero
    pub replaced_day_reward: TokenAmount,
    /// The original `SealedSectorCID`, only gets set on the first `ReplicaUpdate`
    pub sector_key_cid: Option<Cid>,
    // Flag for QA power mechanism introduced in fip 0045
    pub simple_qa_power: bool,
}
