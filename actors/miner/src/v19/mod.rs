// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_bitfield::BitField;
use fvm_shared4::bigint::BigInt;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::*;
use fvm_shared4::sector::SectorSize;
use fvm_shared4::{METHOD_CONSTRUCTOR, MethodNum};
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
    //ExtendSectorExpiration = 8, // Deprecated
    TerminateSectors = 9,
    DeclareFaults = 10,
    DeclareFaultsRecovered = 11,
    OnDeferredCronEvent = 12,
    CheckSectorProven = 13,
    ApplyRewards = 14,
    ReportConsensusFault = 15,
    WithdrawBalance = 16,
    InternalSectorSetupForPreseal = 17,
    ChangeMultiaddrs = 18,
    CompactPartitions = 19,
    CompactSectorNumbers = 20,
    ConfirmChangeWorkerAddress = 21,
    RepayDebt = 22,
    ChangeOwnerAddress = 23,
    DisputeWindowedPoSt = 24,
    //PreCommitSectorBatch = 25, // Deprecated
    // ProveCommitAggregate = 26, // Deprecated
    // ProveReplicaUpdates = 27, // Deprecated
    PreCommitSectorBatch2 = 28,
    //ProveReplicaUpdates2 = 29, // Deprecated
    ChangeBeneficiary = 30,
    GetBeneficiary = 31,
    ExtendSectorExpiration2 = 32,
    // MovePartitions = 33,
    ProveCommitSectors3 = 34,
    ProveReplicaUpdates3 = 35,
    ProveCommitSectorsNI = 36,
    UpgradeSectorQuality = 37,
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
    MaxTerminationFeeExported = frc42_dispatch::method_hash!("MaxTerminationFee"),
    InitialPledgeExported = frc42_dispatch::method_hash!("InitialPledge"),
    GenerateSectorLocationExported = frc42_dispatch::method_hash!("GenerateSectorLocation"),
    ValidateSectorStatusExported = frc42_dispatch::method_hash!("ValidateSectorStatus"),
    GetNominalSectorExpirationExported = frc42_dispatch::method_hash!("GetNominalSectorExpiration"),
}

pub const SECTOR_CONTENT_CHANGED: MethodNum = frc42_dispatch::method_hash!("SectorContentChanged");

pub const ERR_BALANCE_INVARIANTS_BROKEN: ExitCode = ExitCode::new(1000);
pub const ERR_NOTIFICATION_SEND_FAILED: ExitCode = ExitCode::new(1001);
pub const ERR_NOTIFICATION_RECEIVER_ABORTED: ExitCode = ExitCode::new(1002);
pub const ERR_NOTIFICATION_RESPONSE_INVALID: ExitCode = ExitCode::new(1003);
pub const ERR_NOTIFICATION_REJECTED: ExitCode = ExitCode::new(1004);

#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedExpirationExtension {
    pub deadline: u64,
    pub partition: u64,
    pub sectors: BitField,
    /// Absent when the declaration rewrites its sectors without moving their expiration.
    pub new_expiration: Option<ChainEpoch>,
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
            new_expiration: Some(e2.new_expiration),
        }
    }
}

impl From<UpgradeSectorQuality> for ValidatedExpirationExtension {
    fn from(uq: UpgradeSectorQuality) -> Self {
        let UpgradeSectorQuality {
            deadline,
            partition,
            sectors,
            new_expiration,
        } = uq;
        Self {
            deadline,
            partition,
            sectors,
            new_expiration,
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

pub fn daily_fee_for_sectors(sectors: &[SectorOnChainInfo]) -> TokenAmount {
    sectors.iter().map(|s| &s.daily_fee).sum()
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
        Err(anyhow::anyhow!(
            "not all sectors are assigned to the partition"
        ))
    }
}
