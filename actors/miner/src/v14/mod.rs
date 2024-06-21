// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_ipld_encoding::RawBytes;
use fvm_shared4::bigint::BigInt;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::deal::DealID;
use fvm_shared4::error::*;
use fvm_shared4::sector::{RegisteredSealProof, RegisteredUpdateProof, SectorNumber, SectorSize};
use fvm_shared4::{MethodNum, METHOD_CONSTRUCTOR};
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
pub use quantize::*;
pub use sector_map::*;
pub use sectors::*;
pub use state::*;
pub use termination::*;
pub use types::*;
pub use vesting_state::*;

// The following errors are particular cases of illegal state.
// They're not expected to ever happen, but if they do, distinguished codes can help us
// diagnose the problem.

#[cfg(feature = "fil-actor")]
fil_actors_shared::v14::wasm_trampoline!(Actor);

mod beneficiary;
mod bitfield_queue;
mod commd;
mod deadline_assignment;
mod deadline_info;
mod deadline_state;
mod deadlines;
mod expiration_queue;
#[doc(hidden)]
pub mod ext;
mod monies;
mod partition_state;
mod policy;
mod quantize;
mod sector_map;
mod sectors;
mod state;
mod termination;
mod types;
mod vesting_state;

/// Storage Miner actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    ControlAddresses = 2,
    ChangeWorkerAddress = 3,
    ChangePeerID = 4,
    SubmitWindowedPoSt = 5,
    //PreCommitSector = 6, // Deprecated
    //ProveCommitSector = 7, // Deprecated
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
    //PreCommitSectorBatch = 25, // Deprecated
    ProveCommitAggregate = 26,
    ProveReplicaUpdates = 27,
    PreCommitSectorBatch2 = 28,
    //ProveReplicaUpdates2 = 29, // Deprecated
    ChangeBeneficiary = 30,
    GetBeneficiary = 31,
    ExtendSectorExpiration2 = 32,
    // MovePartitions = 33,
    ProveCommitSectors3 = 34,
    ProveReplicaUpdates3 = 35,
    ProveCommitSectorsNI = 36,
    // Method numbers derived from FRC-0042 standards
    ChangeWorkerAddressExported = frc42_dispatch::method_hash!("ChangeWorkerAddress"),
    ChangePeerIDExported = frc42_dispatch::method_hash!("ChangePeerID"),
    WithdrawBalanceExported = frc42_dispatch::method_hash!("WithdrawBalance"),
    ChangeMultiaddrsExported = frc42_dispatch::method_hash!("ChangeMultiaddrs"),
    ConfirmChangeWorkerAddressExported = frc42_dispatch::method_hash!("ConfirmChangeWorkerAddress"),
    RepayDebtExported = frc42_dispatch::method_hash!("RepayDebt"),
    ChangeOwnerAddressExported = frc42_dispatch::method_hash!("ChangeOwnerAddress"),
    ChangeBeneficiaryExported = frc42_dispatch::method_hash!("ChangeBeneficiary"),
    GetBeneficiaryExported = frc42_dispatch::method_hash!("GetBeneficiary"),
    GetOwnerExported = frc42_dispatch::method_hash!("GetOwner"),
    IsControllingAddressExported = frc42_dispatch::method_hash!("IsControllingAddress"),
    GetSectorSizeExported = frc42_dispatch::method_hash!("GetSectorSize"),
    GetAvailableBalanceExported = frc42_dispatch::method_hash!("GetAvailableBalance"),
    GetVestingFundsExported = frc42_dispatch::method_hash!("GetVestingFunds"),
    GetPeerIDExported = frc42_dispatch::method_hash!("GetPeerID"),
    GetMultiaddrsExported = frc42_dispatch::method_hash!("GetMultiaddrs"),
}

pub const SECTOR_CONTENT_CHANGED: MethodNum = frc42_dispatch::method_hash!("SectorContentChanged");

pub const ERR_BALANCE_INVARIANTS_BROKEN: ExitCode = ExitCode::new(1000);
pub const ERR_NOTIFICATION_SEND_FAILED: ExitCode = ExitCode::new(1001);
pub const ERR_NOTIFICATION_RECEIVER_ABORTED: ExitCode = ExitCode::new(1002);
pub const ERR_NOTIFICATION_RESPONSE_INVALID: ExitCode = ExitCode::new(1003);
pub const ERR_NOTIFICATION_REJECTED: ExitCode = ExitCode::new(1004);

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
    pub replica_proof: RawBytes,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedExpirationExtension {
    pub deadline: u64,
    pub partition: u64,
    pub sectors: BitField,
    pub new_expiration: ChainEpoch,
}

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

pub struct SectorPiecesActivationInput {
    pub piece_manifests: Vec<PieceActivationManifest>,
    pub sector_expiry: ChainEpoch,
    pub sector_number: SectorNumber,
    pub sector_type: RegisteredSealProof,
    pub expected_commd: Option<CompactCommD>,
}

// Inputs for activating builtin market deals for one sector
#[derive(Debug, Clone)]
pub struct DealsActivationInput {
    pub deal_ids: Vec<DealID>,
    pub sector_expiry: ChainEpoch,
    pub sector_number: SectorNumber,
    pub sector_type: RegisteredSealProof,
}

impl From<SectorPreCommitOnChainInfo> for DealsActivationInput {
    fn from(pci: SectorPreCommitOnChainInfo) -> DealsActivationInput {
        DealsActivationInput {
            deal_ids: pci.info.deal_ids,
            sector_expiry: pci.info.expiration,
            sector_number: pci.info.sector_number,
            sector_type: pci.info.seal_proof,
        }
    }
}

impl From<&UpdateAndSectorInfo<'_>> for DealsActivationInput {
    fn from(usi: &UpdateAndSectorInfo) -> DealsActivationInput {
        DealsActivationInput {
            sector_number: usi.sector_info.sector_number,
            sector_expiry: usi.sector_info.expiration,
            deal_ids: usi.update.deals.clone(),
            sector_type: usi.sector_info.seal_proof,
        }
    }
}

// Track information needed to update a sector info's data during ProveReplicaUpdate
#[derive(Clone, Debug)]
struct UpdateAndSectorInfo<'a> {
    update: &'a ReplicaUpdateInner,
    sector_info: &'a SectorOnChainInfo,
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
