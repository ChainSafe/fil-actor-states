// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_shared3::METHOD_CONSTRUCTOR;
use fvm_shared3::bigint::BigInt;
use fvm_shared3::clock::ChainEpoch;
use fvm_shared3::deal::DealID;
use fvm_shared3::sector::*;
use num_derive::FromPrimitive;

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

/// ReplicaUpdate param with Option<Cid> for CommD
/// None means unknown
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

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedExpirationExtension {
    pub deadline: u64,
    pub partition: u64,
    pub sectors: BitField,
    pub new_expiration: ChainEpoch,
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

pub fn power_for_sector(sector_size: SectorSize, sector: &SectorOnChainInfo) -> PowerPair {
    PowerPair {
        raw: BigInt::from(sector_size as u64),
        qa: qa_power_for_sector(sector_size, sector),
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
