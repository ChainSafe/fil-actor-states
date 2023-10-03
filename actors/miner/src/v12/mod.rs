// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_shared3::bigint::BigInt;
use fvm_shared3::clock::ChainEpoch;
use fvm_shared3::deal::DealID;
use fvm_shared3::econ::TokenAmount;
use fvm_shared3::error::*;
use fvm_shared3::sector::*;
use fvm_shared3::smooth::FilterEstimate;
use fvm_shared3::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use std::collections::BTreeMap;

pub use beneficiary::*;
pub use bitfield_queue::*;
pub use commd::*;
pub use deadline_assignment::*;
pub use deadline_info::*;
pub use deadline_state::*;
pub use deadlines::*;
pub use expiration_queue::*;
pub use monies::*;
pub use partition_state::*;
pub use policy::*;
pub use sector_map::*;
pub use sectors::*;
pub use state::*;
pub use termination::*;
pub use types::*;
pub use vesting_state::*;

// The following errors are particular cases of illegal state.
// They're not expected to ever happen, but if they do, distinguished codes can help us
// diagnose the problem.

mod beneficiary;
mod bitfield_queue;
mod commd;
mod deadline_assignment;
mod deadline_info;
mod deadline_state;
mod deadlines;
mod expiration_queue;
mod monies;
mod partition_state;
mod policy;
mod sector_map;
mod sectors;
mod state;
mod termination;
mod types;
mod vesting_state;

// The first 1000 actor-specific codes are left open for user error, i.e. things that might
// actually happen without programming error in the actor code.

// * Updated to specs-actors commit: 17d3c602059e5c48407fb3c34343da87e6ea6586 (v0.9.12)

/// Storage Miner actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    ControlAddresses = 2,
    ChangeWorkerAddress = 3,
    ChangePeerID = 4,
    SubmitWindowedPoSt = 5,
    PreCommitSector = 6,
    ProveCommitSector = 7,
    ExtendSectorExpiration = 8,
    TerminateSectors = 9,
    DeclareFaults = 10,
    DeclareFaultsRecovered = 11,
    OnDeferredCronEvent = 12,
    CheckSectorProven = 13,
    ApplyRewards = 14,
    ReportConsensusFault = 15,
    WithdrawBalance = 16,
    ConfirmSectorProofsValid = 17,
    ChangeMultiaddrs = 18,
    CompactPartitions = 19,
    CompactSectorNumbers = 20,
    ConfirmChangeWorkerAddress = 21,
    RepayDebt = 22,
    ChangeOwnerAddress = 23,
    DisputeWindowedPoSt = 24,
    PreCommitSectorBatch = 25,
    ProveCommitAggregate = 26,
    ProveReplicaUpdates = 27,
    PreCommitSectorBatch2 = 28,
    ProveReplicaUpdates2 = 29,
    ChangeBeneficiary = 30,
    GetBeneficiary = 31,
    ExtendSectorExpiration2 = 32,
    MovePartitions = 33,
    // Method numbers derived from FRC-0042 standards
    ChangeWorkerAddressExported = frc42_macros::method_hash!("ChangeWorkerAddress"),
    ChangePeerIDExported = frc42_macros::method_hash!("ChangePeerID"),
    WithdrawBalanceExported = frc42_macros::method_hash!("WithdrawBalance"),
    ChangeMultiaddrsExported = frc42_macros::method_hash!("ChangeMultiaddrs"),
    ConfirmChangeWorkerAddressExported = frc42_macros::method_hash!("ConfirmChangeWorkerAddress"),
    RepayDebtExported = frc42_macros::method_hash!("RepayDebt"),
    ChangeOwnerAddressExported = frc42_macros::method_hash!("ChangeOwnerAddress"),
    ChangeBeneficiaryExported = frc42_macros::method_hash!("ChangeBeneficiary"),
    GetBeneficiaryExported = frc42_macros::method_hash!("GetBeneficiary"),
    GetOwnerExported = frc42_macros::method_hash!("GetOwner"),
    IsControllingAddressExported = frc42_macros::method_hash!("IsControllingAddress"),
    GetSectorSizeExported = frc42_macros::method_hash!("GetSectorSize"),
    GetAvailableBalanceExported = frc42_macros::method_hash!("GetAvailableBalance"),
    GetVestingFundsExported = frc42_macros::method_hash!("GetVestingFunds"),
    GetPeerIDExported = frc42_macros::method_hash!("GetPeerID"),
    GetMultiaddrsExported = frc42_macros::method_hash!("GetMultiaddrs"),
}

pub const ERR_BALANCE_INVARIANTS_BROKEN: ExitCode = ExitCode::new(1000);

#[derive(Debug, PartialEq, Clone)]
struct SectorPreCommitInfoInner {
    pub seal_proof: RegisteredSealProof,
    pub sector_number: SectorNumber,
    /// CommR
    pub sealed_cid: Cid,
    pub seal_rand_epoch: ChainEpoch,
    pub deal_ids: Vec<DealID>,
    pub expiration: ChainEpoch,
    /// CommD
    pub unsealed_cid: Option<CompactCommD>,
}

/// ReplicaUpdate param with Option<Cid> for CommD
/// None means unknown
#[derive(Debug, Clone)]
pub struct ReplicaUpdateInner {
    pub sector_number: SectorNumber,
    pub deadline: u64,
    pub partition: u64,
    pub new_sealed_cid: Cid,
    /// None means unknown
    pub new_unsealed_cid: Option<Cid>,
    pub deals: Vec<DealID>,
    pub update_proof_type: RegisteredUpdateProof,
    pub replica_proof: Vec<u8>,
}

enum ExtensionKind {
    // handle only legacy sectors
    ExtendCommittmentLegacy,
    // handle both Simple QAP and legacy sectors
    ExtendCommittment,
}

// ExtendSectorExpiration param
struct ExtendExpirationsInner {
    extensions: Vec<ValidatedExpirationExtension>,
    // Map from sector being extended to (check, maintain)
    // `check` is the space of active claims, checked to ensure all claims are checked
    // `maintain` is the space of claims to maintain
    // maintain <= check with equality in the case no claims are dropped
    claims: Option<BTreeMap<SectorNumber, (u64, u64)>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedExpirationExtension {
    pub deadline: u64,
    pub partition: u64,
    pub sectors: BitField,
    pub new_expiration: ChainEpoch,
}

#[allow(clippy::too_many_arguments)] // validate mut prevents implementing From
impl From<ExpirationExtension2> for ValidatedExpirationExtension {
    fn from(e2: ExpirationExtension2) -> Self {
        let mut sectors = BitField::new();
        for sc in e2.sectors_with_claims {
            sectors.set(sc.sector_number)
        }
        sectors |= &e2.sectors;

        Self {
            deadline: e2.deadline,
            partition: e2.partition,
            sectors,
            new_expiration: e2.new_expiration,
        }
    }
}

struct SectorSealProofInput {
    pub registered_proof: RegisteredSealProof,
    pub sector_number: SectorNumber,
    pub randomness: SealRandomness,
    pub interactive_randomness: InteractiveSealRandomness,
    pub sealed_cid: Cid,   // Commr
    pub unsealed_cid: Cid, // Commd
}

impl SectorSealProofInput {
    fn to_seal_verify_info(&self, miner_actor_id: u64, proof: Vec<u8>) -> SealVerifyInfo {
        SealVerifyInfo {
            registered_proof: self.registered_proof,
            sector_id: SectorID {
                miner: miner_actor_id,
                number: self.sector_number,
            },
            deal_ids: vec![], // unused by the proofs api so this is safe to leave empty
            randomness: self.randomness.clone(),
            interactive_randomness: self.interactive_randomness.clone(),
            proof,
            sealed_cid: self.sealed_cid,
            unsealed_cid: self.unsealed_cid,
        }
    }

    fn to_aggregate_seal_verify_info(&self) -> AggregateSealVerifyInfo {
        AggregateSealVerifyInfo {
            sector_number: self.sector_number,
            randomness: self.randomness.clone(),
            interactive_randomness: self.interactive_randomness.clone(),
            sealed_cid: self.sealed_cid,
            unsealed_cid: self.unsealed_cid,
        }
    }
}

pub fn power_for_sector(sector_size: SectorSize, sector: &SectorOnChainInfo) -> PowerPair {
    PowerPair {
        raw: BigInt::from(sector_size as u64),
        qa: qa_power_for_sector(sector_size, sector),
    }
}

/// Validates that a partition contains the given sectors.
fn validate_partition_contains_sectors(
    partition: &Partition,
    sectors: &BitField,
) -> anyhow::Result<()> {
    // Check that the declared sectors are actually assigned to the partition.
    if partition.sectors.contains_all(sectors) {
        Ok(())
    } else {
        Err(anyhow!("not all sectors are assigned to the partition"))
    }
}

/// Returns the sum of the raw byte and quality-adjusted power for sectors.
pub fn power_for_sectors(sector_size: SectorSize, sectors: &[SectorOnChainInfo]) -> PowerPair {
    let qa = sectors
        .iter()
        .map(|s| qa_power_for_sector(sector_size, s))
        .sum();

    PowerPair {
        raw: BigInt::from(sector_size as u64) * BigInt::from(sectors.len()),
        qa,
    }
}

// Inputs for activating builtin market deals for one sector
#[derive(Debug, Clone)]
pub struct DealsActivationInput {
    pub deal_ids: Vec<DealID>,
    pub sector_expiry: ChainEpoch,
    pub sector_number: SectorNumber,
    pub sector_type: RegisteredSealProof,
}

// Inputs for activating builtin market deals for one sector
// and optionally confirming CommD for this sector matches expectation
struct DataActivationInput {
    info: DealsActivationInput,
    expected_commd: Option<Cid>,
}

impl From<SectorPreCommitOnChainInfo> for DataActivationInput {
    fn from(pci: SectorPreCommitOnChainInfo) -> DataActivationInput {
        DataActivationInput {
            info: DealsActivationInput {
                deal_ids: pci.info.deal_ids,
                sector_expiry: pci.info.expiration,
                sector_number: pci.info.sector_number,
                sector_type: pci.info.seal_proof,
            },
            expected_commd: None, // CommD checks are always performed at precommit time
        }
    }
}

impl From<UpdateAndSectorInfo<'_>> for DataActivationInput {
    fn from(usi: UpdateAndSectorInfo) -> DataActivationInput {
        DataActivationInput {
            info: DealsActivationInput {
                sector_number: usi.sector_info.sector_number,
                sector_expiry: usi.sector_info.expiration,
                deal_ids: usi.update.deals.clone(),
                sector_type: usi.sector_info.seal_proof,
            },
            expected_commd: usi.update.new_unsealed_cid,
        }
    }
}

// Data activation results for one sector
#[derive(Clone)]
struct DataActivationOutput {
    pub unverified_space: BigInt,
    pub verified_space: BigInt,
    // None indicates either no deals or computation was not requested.
    pub unsealed_cid: Option<Cid>,
}

// Track information needed to update a sector info's data during ProveReplicaUpdate
#[derive(Clone, Debug)]
struct UpdateAndSectorInfo<'a> {
    update: &'a ReplicaUpdateInner,
    sector_info: SectorOnChainInfo,
}

// Inputs to proof verification and state update for a single sector replica update.
struct ReplicaUpdateInputs<'a> {
    deadline: u64,
    partition: u64,
    sector_info: &'a SectorOnChainInfo,
    proof_inputs: ReplicaUpdateInfo,
    activated_data: ReplicaUpdateActivatedData,
}

// Summary of activated data for a replica update.
struct ReplicaUpdateActivatedData {
    seal_cid: Cid,
    deals: Vec<DealID>,
    unverified_space: BigInt,
    verified_space: BigInt,
}

// Network inputs to calculation of sector pledge and associated parameters.
struct NetworkPledgeInputs {
    pub network_qap: FilterEstimate,
    pub network_baseline: StoragePower,
    pub circulating_supply: TokenAmount,
    pub epoch_reward: FilterEstimate,
}
