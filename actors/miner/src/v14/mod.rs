// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::cmp::max;
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Error};
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_encoding::{BytesDe, RawBytes};
use fvm_shared4::address::{Address, Payload, Protocol};
use fvm_shared4::bigint::{BigInt, Integer};
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::deal::DealID;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::*;
use fvm_shared4::piece::PieceInfo;
use fvm_shared4::randomness::*;
use fvm_shared4::sector::{
    AggregateSealVerifyInfo, AggregateSealVerifyProofAndInfos, InteractiveSealRandomness,
    PoStProof, RegisteredAggregateProof, RegisteredPoStProof, RegisteredSealProof,
    RegisteredUpdateProof, ReplicaUpdateInfo, SealRandomness, SealVerifyInfo, SectorID, SectorInfo,
    SectorNumber, SectorSize, StoragePower, WindowPoStVerifyInfo,
};
use fvm_shared4::{ActorID, MethodNum, METHOD_CONSTRUCTOR, METHOD_SEND};
use itertools::Itertools;
use log::{error, info, warn};
use num_derive::FromPrimitive;
use num_traits::{Signed, Zero};

pub use beneficiary::*;
pub use bitfield_queue::*;
pub use commd::*;
pub use deadline_assignment::*;
pub use deadline_info::*;
pub use deadline_state::*;
pub use deadlines::*;
pub use expiration_queue::*;
use fil_actors_shared::v14::cbor::{serialize, serialize_vec};
use fil_actors_shared::v14::reward::{FilterEstimate, ThisEpochRewardReturn};
use fil_actors_shared::v14::runtime::builtins::Type;
use fil_actors_shared::v14::runtime::policy_constants::MAX_SECTOR_NUMBER;
use fil_actors_shared::v14::runtime::{DomainSeparationTag, Policy, Runtime};
use fil_actors_shared::v14::{
    deserialize_block, extract_send_result, ActorContext,
    ActorDowncast, ActorError, AsActorError, BatchReturn, BatchReturnGen, DealWeight,
    BURNT_FUNDS_ACTOR_ADDR, REWARD_ACTOR_ADDR, STORAGE_MARKET_ACTOR_ADDR,
    STORAGE_POWER_ACTOR_ADDR, SYSTEM_ACTOR_ADDR, VERIFIED_REGISTRY_ACTOR_ADDR,
};
use fil_actors_shared::actor_error_v14;
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

use crate::v14::ext::market::NO_ALLOCATION_ID;

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
mod emit;
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
    pub replica_proof: RawBytes,
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

fn validate_legacy_extension_declarations(
    extensions: &[ExpirationExtension],
    policy: &Policy,
) -> Result<ExtendExpirationsInner, ActorError> {
    let vec_validated = extensions
        .iter()
        .map(|decl| {
            if decl.deadline >= policy.wpost_period_deadlines {
                return Err(actor_error_v14!(
                    illegal_argument,
                    "deadline {} not in range 0..{}",
                    decl.deadline,
                    policy.wpost_period_deadlines
                ));
            }

            Ok(ValidatedExpirationExtension {
                deadline: decl.deadline,
                partition: decl.partition,
                sectors: decl.sectors.clone(),
                new_expiration: decl.new_expiration,
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(ExtendExpirationsInner { extensions: vec_validated, claims: None })
}

fn validate_extension_declarations(
    rt: &impl Runtime,
    extensions: Vec<ExpirationExtension2>,
) -> Result<ExtendExpirationsInner, ActorError> {
    let mut claim_space_by_sector = BTreeMap::<SectorNumber, (u64, u64)>::new();

    for decl in &extensions {
        let policy = rt.policy();
        if decl.deadline >= policy.wpost_period_deadlines {
            return Err(actor_error_v14!(
                illegal_argument,
                "deadline {} not in range 0..{}",
                decl.deadline,
                policy.wpost_period_deadlines
            ));
        }

        for sc in &decl.sectors_with_claims {
            let mut drop_claims = sc.drop_claims.clone();
            let mut all_claim_ids = sc.maintain_claims.clone();
            all_claim_ids.append(&mut drop_claims);
            let claims = get_claims(rt, &all_claim_ids)
                .with_context(|| format!("failed to get claims for sector {}", sc.sector_number))?;
            let first_drop = sc.maintain_claims.len();

            for (i, claim) in claims.iter().enumerate() {
                // check provider and sector matches
                if claim.provider != rt.message().receiver().id().unwrap() {
                    return Err(actor_error_v14!(illegal_argument, "failed to validate declaration sector={}, claim={}, expected claim provider to be {} but found {} ", sc.sector_number, all_claim_ids[i], rt.message().receiver().id().unwrap(), claim.provider));
                }
                if claim.sector != sc.sector_number {
                    return Err(actor_error_v14!(illegal_argument, "failed to validate declaration sector={}, claim={} expected claim sector number to be {} but found {} ", sc.sector_number, all_claim_ids[i], sc.sector_number, claim.sector));
                }

                // If we are not dropping check expiration does not exceed term max
                let mut maintain_delta: u64 = 0;
                if i < first_drop {
                    if decl.new_expiration > claim.term_start + claim.term_max {
                        return Err(actor_error_v14!(forbidden, "failed to validate declaration sector={}, claim={} claim only allows extension to {} but declared new expiration is {}", sc.sector_number, sc.maintain_claims[i], claim.term_start + claim.term_max, decl.new_expiration));
                    }
                    maintain_delta = claim.size.0
                }

                claim_space_by_sector
                    .entry(sc.sector_number)
                    .and_modify(|(check, maintain)| {
                        *check += claim.size.0;
                        *maintain += maintain_delta;
                    })
                    .or_insert((claim.size.0, maintain_delta));
            }
        }
    }
    Ok(ExtendExpirationsInner {
        extensions: extensions.into_iter().map(|e2| e2.into()).collect(),
        claims: Some(claim_space_by_sector),
    })
}

#[allow(clippy::too_many_arguments)]
fn extend_sector_committment(
    policy: &Policy,
    curr_epoch: ChainEpoch,
    reward_stats: &ThisEpochRewardReturn,
    power_stats: &ext::power::CurrentTotalPowerReturn,
    new_expiration: ChainEpoch,
    sector: &SectorOnChainInfo,
    sector_size: SectorSize,
    claim_space_by_sector: &BTreeMap<SectorNumber, (u64, u64)>,
) -> Result<SectorOnChainInfo, ActorError> {
    validate_extended_expiration(policy, curr_epoch, new_expiration, sector)?;

    // all simple_qa_power sectors with VerifiedDealWeight > 0 MUST check all claims
    if sector.flags.contains(SectorOnChainInfoFlags::SIMPLE_QA_POWER) {
        extend_simple_qap_sector(
            policy,
            new_expiration,
            curr_epoch,
            reward_stats,
            power_stats,
            sector,
            sector_size,
            claim_space_by_sector,
        )
    } else {
        extend_non_simple_qap_sector(new_expiration, curr_epoch, sector)
    }
}

fn extend_sector_committment_legacy(
    policy: &Policy,
    curr_epoch: ChainEpoch,
    new_expiration: ChainEpoch,
    sector: &SectorOnChainInfo,
) -> Result<SectorOnChainInfo, ActorError> {
    validate_extended_expiration(policy, curr_epoch, new_expiration, sector)?;

    // it is an error to do legacy sector expiration on simple-qa power sectors with deal weight
    if sector.flags.contains(SectorOnChainInfoFlags::SIMPLE_QA_POWER)
        && (sector.verified_deal_weight > BigInt::zero() || sector.deal_weight > BigInt::zero())
    {
        return Err(actor_error_v14!(
            forbidden,
            "cannot use legacy sector extension for simple qa power with deal weight {}",
            sector.sector_number
        ));
    }
    extend_non_simple_qap_sector(new_expiration, curr_epoch, sector)
}

fn validate_extended_expiration(
    policy: &Policy,
    curr_epoch: ChainEpoch,
    new_expiration: ChainEpoch,
    sector: &SectorOnChainInfo,
) -> Result<(), ActorError> {
    if !can_extend_seal_proof_type(sector.seal_proof) {
        return Err(actor_error_v14!(
            forbidden,
            "cannot extend expiration for sector {} with unsupported \
            seal type {:?}",
            sector.sector_number,
            sector.seal_proof
        ));
    }
    // This can happen if the sector should have already expired, but hasn't
    // because the end of its deadline hasn't passed yet.
    if sector.expiration < curr_epoch {
        return Err(actor_error_v14!(
            forbidden,
            "cannot extend expiration for expired sector {} at {}",
            sector.sector_number,
            sector.expiration
        ));
    }

    if new_expiration < sector.expiration {
        return Err(actor_error_v14!(
            illegal_argument,
            "cannot reduce sector {} expiration to {} from {}",
            sector.sector_number,
            new_expiration,
            sector.expiration
        ));
    }

    validate_expiration(policy, curr_epoch, sector.activation, new_expiration, sector.seal_proof)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn extend_simple_qap_sector(
    policy: &Policy,
    new_expiration: ChainEpoch,
    curr_epoch: ChainEpoch,
    reward_stats: &ThisEpochRewardReturn,
    power_stats: &ext::power::CurrentTotalPowerReturn,
    sector: &SectorOnChainInfo,
    sector_size: SectorSize,
    claim_space_by_sector: &BTreeMap<SectorNumber, (u64, u64)>,
) -> Result<SectorOnChainInfo, ActorError> {
    let mut new_sector = sector.clone();

    new_sector.expiration = new_expiration;
    new_sector.power_base_epoch = curr_epoch;
    let old_duration = sector.expiration - sector.power_base_epoch;
    let new_duration = new_sector.expiration - new_sector.power_base_epoch;

    // Update the non-verified deal weights. This won't change power, it'll just keep it the same
    // relative to the updated power base epoch.
    if sector.deal_weight.is_positive() {
        // (old_deal_weight) / old_duration -> old_space
        // old_space * (old_expiration - curr_epoch) -> remaining spacetime in the deals.
        new_sector.deal_weight =
            &sector.deal_weight * (sector.expiration - curr_epoch) / old_duration;
    }

    // Update the verified deal weights, and pledge if necessary.
    if sector.verified_deal_weight.is_positive() {
        let old_verified_deal_space = &sector.verified_deal_weight / old_duration;
        let (expected_verified_deal_space, new_verified_deal_space) = match claim_space_by_sector
            .get(&sector.sector_number)
        {
            None => {
                return Err(actor_error_v14!(
                        illegal_argument,
                        "claim missing from declaration for sector {} with non-zero verified deal weight {}",
                        sector.sector_number,
                        &sector.verified_deal_weight
                    ));
            }
            Some(space) => space,
        };
        // claims must be completely accounted for
        if BigInt::from(*expected_verified_deal_space as i64) != old_verified_deal_space {
            return Err(actor_error_v14!(illegal_argument, "declared verified deal space in claims ({}) does not match verified deal space ({}) for sector {}", expected_verified_deal_space, old_verified_deal_space, sector.sector_number));
        }
        // claim dropping is restricted to extensions at the end of a sector's life

        let dropping_claims = expected_verified_deal_space != new_verified_deal_space;
        if dropping_claims && sector.expiration - curr_epoch > policy.end_of_life_claim_drop_period
        {
            return Err(actor_error_v14!(
                forbidden,
                "attempt to drop claims with {} epochs > end of life claim drop period {} remaining",
                sector.expiration - curr_epoch,
                policy.end_of_life_claim_drop_period
            ));
        }

        new_sector.verified_deal_weight = BigInt::from(*new_verified_deal_space) * new_duration;

        // We only bother updating the expected_day_reward, expected_storage_pledge, and replaced_day_reward
        //  for verified deals, as it can increase power.
        let qa_pow = qa_power_for_weight(
            sector_size,
            new_duration,
            &new_sector.deal_weight,
            &new_sector.verified_deal_weight,
        );
        new_sector.expected_day_reward = expected_reward_for_power(
            &reward_stats.this_epoch_reward_smoothed,
            &power_stats.quality_adj_power_smoothed,
            &qa_pow,
            fil_actors_shared::v14::network::EPOCHS_IN_DAY,
        );
        new_sector.expected_storage_pledge = max(
            sector.expected_storage_pledge.clone(),
            expected_reward_for_power(
                &reward_stats.this_epoch_reward_smoothed,
                &power_stats.quality_adj_power_smoothed,
                &qa_pow,
                INITIAL_PLEDGE_PROJECTION_PERIOD,
            ),
        );
        new_sector.replaced_day_reward =
            max(sector.expected_day_reward.clone(), sector.replaced_day_reward.clone());
    }

    Ok(new_sector)
}

fn extend_non_simple_qap_sector(
    new_expiration: ChainEpoch,
    curr_epoch: ChainEpoch,
    sector: &SectorOnChainInfo,
) -> Result<SectorOnChainInfo, ActorError> {
    let mut new_sector = sector.clone();
    // Remove "spent" deal weights for non simple_qa_power sectors with deal weight > 0
    let new_deal_weight = (&sector.deal_weight * (sector.expiration - curr_epoch))
        .div_floor(&BigInt::from(sector.expiration - sector.power_base_epoch));

    let new_verified_deal_weight = (&sector.verified_deal_weight
        * (sector.expiration - curr_epoch))
        .div_floor(&BigInt::from(sector.expiration - sector.power_base_epoch));

    new_sector.expiration = new_expiration;
    new_sector.deal_weight = new_deal_weight;
    new_sector.verified_deal_weight = new_verified_deal_weight;
    new_sector.power_base_epoch = curr_epoch;

    Ok(new_sector)
}

// Validates a list of replica update requests and parallel sector infos.
// Returns all pairs of update and sector info, even those that fail validation.
// The proof verification inputs are needed as witnesses to verify an aggregate proof to allow
// other, valid, updates to succeed.
#[allow(clippy::too_many_arguments)]
fn validate_replica_updates<'a, BS>(
    updates: &'a [ReplicaUpdateInner],
    sector_infos: &'a [SectorOnChainInfo],
    state: &State,
    sector_size: SectorSize,
    policy: &Policy,
    curr_epoch: ChainEpoch,
    store: BS,
    require_deals: bool,
    all_or_nothing: bool,
) -> Result<(BatchReturn, Vec<UpdateAndSectorInfo<'a>>), ActorError>
where
    BS: Blockstore,
{
    if updates.len() > policy.prove_replica_updates_max_size {
        return Err(actor_error_v14!(
            illegal_argument,
            "too many updates ({} > {})",
            updates.len(),
            policy.prove_replica_updates_max_size
        ));
    }

    let mut sector_numbers = BTreeSet::<SectorNumber>::new();
    let mut validate_one = |update: &ReplicaUpdateInner,
                            sector_info: &SectorOnChainInfo|
     -> Result<(), ActorError> {
        if !sector_numbers.insert(update.sector_number) {
            return Err(actor_error_v14!(
                illegal_argument,
                "skipping duplicate sector {}",
                update.sector_number
            ));
        }

        if update.replica_proof.len() > 4096 {
            return Err(actor_error_v14!(
                illegal_argument,
                "update proof is too large ({}), skipping sector {}",
                update.replica_proof.len(),
                update.sector_number
            ));
        }

        if require_deals && update.deals.is_empty() {
            return Err(actor_error_v14!(
                illegal_argument,
                "must have deals to update, skipping sector {}",
                update.sector_number
            ));
        }

        if update.deals.len() as u64 > sector_deals_max(policy, sector_size) {
            return Err(actor_error_v14!(
                illegal_argument,
                "more deals than policy allows, skipping sector {}",
                update.sector_number
            ));
        }

        if update.deadline >= policy.wpost_period_deadlines {
            return Err(actor_error_v14!(
                illegal_argument,
                "deadline {} not in range 0..{}, skipping sector {}",
                update.deadline,
                policy.wpost_period_deadlines,
                update.sector_number
            ));
        }

        if !is_sealed_sector(&update.new_sealed_cid) {
            return Err(actor_error_v14!(
                illegal_argument,
                "new sealed CID had wrong prefix {}, skipping sector {}",
                update.new_sealed_cid,
                update.sector_number
            ));
        }

        // Disallow upgrading sectors in immutable deadlines.
        if !deadline_is_mutable(
            policy,
            state.current_proving_period_start(policy, curr_epoch),
            update.deadline,
            curr_epoch,
        ) {
            return Err(actor_error_v14!(
                illegal_argument,
                "cannot upgrade sectors in immutable deadline {}, skipping sector {}",
                update.deadline,
                update.sector_number
            ));
        }

        // This inefficiently loads deadline/partition info for each update.
        if !state.check_sector_active(
            &store,
            update.deadline,
            update.partition,
            update.sector_number,
            true,
        )? {
            return Err(actor_error_v14!(
                illegal_argument,
                "sector isn't active, skipping sector {}",
                update.sector_number
            ));
        }

        if (&sector_info.deal_weight + &sector_info.verified_deal_weight) != DealWeight::zero() {
            return Err(actor_error_v14!(
                illegal_argument,
                "cannot update sector with non-zero data, skipping sector {}",
                update.sector_number
            ));
        }

        let expected_proof_type = sector_info
            .seal_proof
            .registered_update_proof()
            .context_code(ExitCode::USR_ILLEGAL_STATE, "couldn't load update proof type")?;
        if update.update_proof_type != expected_proof_type {
            return Err(actor_error_v14!(
                illegal_argument,
                "expected proof type {}, was {}",
                i64::from(expected_proof_type),
                i64::from(update.update_proof_type)
            ));
        }
        Ok(())
    };

    let mut batch = BatchReturnGen::new(updates.len());
    let mut update_sector_infos: Vec<UpdateAndSectorInfo> = Vec::with_capacity(updates.len());
    for (i, (update, sector_info)) in updates.iter().zip(sector_infos).enumerate() {
        // Build update and sector info for all updates, even if they fail validation.
        update_sector_infos.push(UpdateAndSectorInfo { update, sector_info });

        match validate_one(update, sector_info) {
            Ok(_) => {
                batch.add_success();
            }
            Err(e) => {
                let e = e.wrap(format!("invalid update {} while requiring activation success", i));
                info!("{}", e.msg());
                if all_or_nothing {
                    return Err(e);
                }
                batch.add_fail(ExitCode::USR_ILLEGAL_ARGUMENT);
            }
        }
    }
    Ok((batch.gen(), update_sector_infos))
}

fn update_replica_states<BS>(
    rt: &impl Runtime,
    updates_by_deadline: &BTreeMap<u64, Vec<ReplicaUpdateStateInputs>>,
    expected_count: usize,
    sectors: &mut Sectors<BS>,
    sector_size: SectorSize,
) -> Result<(PowerPair, TokenAmount), ActorError>
where
    BS: Blockstore,
{
    let rew = request_current_epoch_block_reward(rt)?;
    let pow = request_current_total_power(rt)?;
    let circulating_supply = rt.total_fil_circ_supply();
    let pledge_inputs = NetworkPledgeInputs {
        network_qap: pow.quality_adj_power_smoothed,
        network_baseline: rew.this_epoch_baseline_power,
        circulating_supply,
        epoch_reward: rew.this_epoch_reward_smoothed,
    };
    let mut power_delta = PowerPair::zero();
    let mut pledge_delta = TokenAmount::zero();

    rt.transaction(|state: &mut State, rt| {
        let mut deadlines = state.load_deadlines(rt.store())?;
        let mut new_sectors = Vec::with_capacity(expected_count);
        // Process updates grouped by deadline.
        for (&dl_idx, updates) in updates_by_deadline {
            let mut deadline = deadlines.load_deadline(rt.store(), dl_idx)?;

            let mut partitions = deadline
                .partitions_amt(rt.store())
                .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                    format!("failed to load partitions for deadline {}", dl_idx)
                })?;

            let quant = state.quant_spec_for_deadline(rt.policy(), dl_idx);

            for update in updates {
                // Compute updated sector info.
                let new_sector_info = update_existing_sector_info(
                    update.sector_info,
                    &update.activated_data,
                    &pledge_inputs,
                    sector_size,
                    rt.curr_epoch(),
                );

                let mut partition = partitions
                    .get(update.partition)
                    .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                        format!(
                            "failed to load deadline {} partition {}",
                            update.deadline, update.partition
                        )
                    })?
                    .cloned()
                    .ok_or_else(|| {
                        actor_error_v14!(
                            not_found,
                            "no such deadline {} partition {}",
                            dl_idx,
                            update.partition
                        )
                    })?;

                // Note: replacing sectors one at a time in each partition is inefficient.
                let (partition_power_delta, partition_pledge_delta) = partition
                    .replace_sectors(
                        rt.store(),
                        std::slice::from_ref(update.sector_info),
                        std::slice::from_ref(&new_sector_info),
                        sector_size,
                        quant,
                    )
                    .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                        format!(
                            "failed to replace sector at deadline {} partition {}",
                            update.deadline, update.partition
                        )
                    })?;

                power_delta += &partition_power_delta;
                pledge_delta += &partition_pledge_delta;

                partitions.set(update.partition, partition).with_context_code(
                    ExitCode::USR_ILLEGAL_STATE,
                    || {
                        format!(
                            "failed to save deadline {} partition {}",
                            update.deadline, update.partition
                        )
                    },
                )?;

                new_sectors.push(new_sector_info);
            } // End loop over declarations in one deadline.

            deadline.partitions =
                partitions.flush().with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                    format!("failed to save partitions for deadline {}", dl_idx)
                })?;

            deadlines
                .update_deadline(rt.policy(), rt.store(), dl_idx, &deadline)
                .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                    format!("failed to save deadline {}", dl_idx)
                })?;
        } // End loop over deadlines

        if new_sectors.len() != expected_count {
            return Err(actor_error_v14!(
                illegal_state,
                "unexpected new_sectors len {} != {}",
                new_sectors.len(),
                expected_count
            ));
        }

        // Overwrite sector infos.
        sectors.store(new_sectors).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to update sector infos")
        })?;

        state.sectors = sectors.amt.flush().map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save sectors")
        })?;
        state.save_deadlines(rt.store(), deadlines).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save deadlines")
        })?;

        // Update pledge.
        let current_balance = rt.current_balance();
        if pledge_delta.is_positive() {
            let unlocked_balance = state.get_unlocked_balance(&current_balance).map_err(|e| {
                actor_error_v14!(illegal_state, "failed to calculate unlocked balance: {}", e)
            })?;
            if unlocked_balance < pledge_delta {
                return Err(actor_error_v14!(
                    insufficient_funds,
                    "insufficient funds for aggregate initial pledge requirement {}, available: {}",
                    pledge_delta,
                    unlocked_balance
                ));
            }
        }

        state
            .add_initial_pledge(&pledge_delta)
            .map_err(|e| actor_error_v14!(illegal_state, "failed to add initial pledge: {}", e))?;

        state.check_balance_invariants(&current_balance).map_err(balance_invariants_broken)?;
        Ok(())
    })?;
    Ok((power_delta, pledge_delta))
}

// Builds a new sector info representing newly activated data in an existing sector.
fn update_existing_sector_info(
    sector_info: &SectorOnChainInfo,
    activated_data: &ReplicaUpdateActivatedData,
    pledge_inputs: &NetworkPledgeInputs,
    sector_size: SectorSize,
    curr_epoch: ChainEpoch,
) -> SectorOnChainInfo {
    let mut new_sector_info = sector_info.clone();

    new_sector_info.flags.set(SectorOnChainInfoFlags::SIMPLE_QA_POWER, true);
    new_sector_info.sealed_cid = activated_data.seal_cid;
    new_sector_info.sector_key_cid = match new_sector_info.sector_key_cid {
        None => Some(sector_info.sealed_cid),
        Some(x) => Some(x),
    };

    new_sector_info.power_base_epoch = curr_epoch;

    let duration = new_sector_info.expiration - new_sector_info.power_base_epoch;

    new_sector_info.deal_weight = activated_data.unverified_space.clone() * duration;
    new_sector_info.verified_deal_weight = activated_data.verified_space.clone() * duration;

    // compute initial pledge
    let qa_pow = qa_power_for_weight(
        sector_size,
        duration,
        &new_sector_info.deal_weight,
        &new_sector_info.verified_deal_weight,
    );

    new_sector_info.replaced_day_reward =
        max(&sector_info.expected_day_reward, &sector_info.replaced_day_reward).clone();
    new_sector_info.expected_day_reward = expected_reward_for_power(
        &pledge_inputs.epoch_reward,
        &pledge_inputs.network_qap,
        &qa_pow,
        fil_actors_shared::v14::network::EPOCHS_IN_DAY,
    );
    new_sector_info.expected_storage_pledge = max(
        new_sector_info.expected_storage_pledge,
        expected_reward_for_power(
            &pledge_inputs.epoch_reward,
            &pledge_inputs.network_qap,
            &qa_pow,
            INITIAL_PLEDGE_PROJECTION_PERIOD,
        ),
    );

    new_sector_info.initial_pledge = max(
        new_sector_info.initial_pledge,
        initial_pledge_for_power(
            &qa_pow,
            &pledge_inputs.network_baseline,
            &pledge_inputs.epoch_reward,
            &pledge_inputs.network_qap,
            &pledge_inputs.circulating_supply,
        ),
    );
    new_sector_info
}

// Note: We're using the current power+epoch reward, rather than at time of termination.
fn process_early_terminations(
    rt: &impl Runtime,
    reward_smoothed: &FilterEstimate,
    quality_adj_power_smoothed: &FilterEstimate,
) -> Result</* more */ bool, ActorError> {
    let mut terminated_sector_nums = vec![];
    let mut sectors_with_data = vec![];
    let (result, more, penalty, pledge_delta) = rt.transaction(|state: &mut State, rt| {
        let store = rt.store();
        let policy = rt.policy();

        let (result, more) = state
            .pop_early_terminations(
                policy,
                store,
                policy.addressed_partitions_max,
                policy.addressed_sectors_max,
            )
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to pop early terminations")?;

        // Nothing to do, don't waste any time.
        // This can happen if we end up processing early terminations
        // before the cron callback fires.
        if result.is_empty() {
            info!("no early terminations (maybe cron callback hasn't happened yet?)");
            return Ok((result, more, TokenAmount::zero(), TokenAmount::zero()));
        }

        let info = get_miner_info(rt.store(), state)?;
        let sectors = Sectors::load(store, &state.sectors).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load sectors array")
        })?;

        let mut total_initial_pledge = TokenAmount::zero();
        let mut total_penalty = TokenAmount::zero();

        for (epoch, sector_numbers) in result.iter() {
            let sectors = sectors
                .load_sector(sector_numbers)
                .map_err(|e| e.wrap("failed to load sector infos"))?;

            for sector in &sectors {
                total_initial_pledge += &sector.initial_pledge;
                let sector_power = qa_power_for_sector(info.sector_size, sector);
                terminated_sector_nums.push(sector.sector_number);
                total_penalty += pledge_penalty_for_termination(
                    &sector.expected_day_reward,
                    epoch - sector.power_base_epoch,
                    &sector.expected_storage_pledge,
                    quality_adj_power_smoothed,
                    &sector_power,
                    reward_smoothed,
                    &sector.replaced_day_reward,
                    sector.power_base_epoch - sector.activation,
                );
                if sector.deal_weight.is_positive() || sector.verified_deal_weight.is_positive() {
                    sectors_with_data.push(sector.sector_number);
                }
            }
        }

        // Apply penalty (add to fee debt)
        state
            .apply_penalty(&total_penalty)
            .map_err(|e| actor_error_v14!(illegal_state, "failed to apply penalty: {}", e))?;

        // Remove pledge requirement.
        let mut pledge_delta = -total_initial_pledge;
        state.add_initial_pledge(&pledge_delta).map_err(|e| {
            actor_error_v14!(illegal_state, "failed to add initial pledge {}: {}", pledge_delta, e)
        })?;

        // Use unlocked pledge to pay down outstanding fee debt
        let (penalty_from_vesting, penalty_from_balance) = state
            .repay_partial_debt_in_priority_order(
                rt.store(),
                rt.curr_epoch(),
                &rt.current_balance(),
            )
            .map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to repay penalty")
            })?;

        let penalty = &penalty_from_vesting + penalty_from_balance;
        pledge_delta -= penalty_from_vesting;

        Ok((result, more, penalty, pledge_delta))
    })?;

    // We didn't do anything, abort.
    if result.is_empty() {
        info!("no early terminations");
        return Ok(more);
    }

    // Burn penalty.
    log::debug!(
        "storage provider {} penalized {} for sector termination",
        rt.message().receiver(),
        penalty
    );
    burn_funds(rt, penalty)?;

    // Return pledge.
    notify_pledge_changed(rt, &pledge_delta)?;

    // Terminate deals.
    let terminated_data = BitField::try_from_bits(sectors_with_data)
        .context_code(ExitCode::USR_ILLEGAL_STATE, "invalid sector number")?;
    request_terminate_deals(rt, rt.curr_epoch(), &terminated_data)?;

    for sector in terminated_sector_nums {
        emit::sector_terminated(rt, sector)?;
    }

    // reschedule cron worker, if necessary.
    Ok(more)
}

/// Invoked at the end of the last epoch for each proving deadline.
fn handle_proving_deadline(
    rt: &impl Runtime,
    reward_smoothed: &FilterEstimate,
    quality_adj_power_smoothed: &FilterEstimate,
) -> Result<(), ActorError> {
    let curr_epoch = rt.curr_epoch();

    let mut had_early_terminations = false;

    let mut power_delta_total = PowerPair::zero();
    let mut penalty_total = TokenAmount::zero();
    let mut pledge_delta_total = TokenAmount::zero();
    let mut continue_cron = false;

    let state: State = rt.transaction(|state: &mut State, rt| {
        let policy = rt.policy();

        // Vesting rewards for a miner are quantized to every 12 hours and we can determine what those "vesting epochs" are.
        // So, only do the vesting here if the current epoch is a "vesting epoch"
        let q = QuantSpec {
            unit: REWARD_VESTING_SPEC.quantization,
            offset: state.proving_period_start,
        };

        if q.quantize_up(curr_epoch) == curr_epoch {
            // Vest locked funds.
            // This happens first so that any subsequent penalties are taken
            // from locked vesting funds before funds free this epoch.
            let newly_vested =
                state.unlock_vested_funds(rt.store(), rt.curr_epoch()).map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to vest funds")
                })?;

            pledge_delta_total -= newly_vested;
        }

        // Process pending worker change if any
        let mut info = get_miner_info(rt.store(), state)?;
        process_pending_worker(&mut info, rt, state)?;

        let deposit_to_burn = state
            .cleanup_expired_pre_commits(policy, rt.store(), rt.curr_epoch())
            .map_err(|e| {
                e.downcast_default(
                    ExitCode::USR_ILLEGAL_STATE,
                    "failed to expire pre-committed sectors",
                )
            })?;
        state
            .apply_penalty(&deposit_to_burn)
            .map_err(|e| actor_error_v14!(illegal_state, "failed to apply penalty: {}", e))?;

        log::debug!(
            "storage provider {} penalized {} for expired pre commits",
            rt.message().receiver(),
            deposit_to_burn
        );

        // Record whether or not we _had_ early terminations in the queue before this method.
        // That way, don't re-schedule a cron callback if one is already scheduled.
        had_early_terminations = have_pending_early_terminations(state);

        let result = state.advance_deadline(policy, rt.store(), rt.curr_epoch()).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to advance deadline")
        })?;

        // Faults detected by this missed PoSt pay no penalty, but sectors that were already faulty
        // and remain faulty through this deadline pay the fault fee.
        let penalty_target = pledge_penalty_for_continued_fault(
            reward_smoothed,
            quality_adj_power_smoothed,
            &result.previously_faulty_power.qa,
        );

        power_delta_total += &result.power_delta;
        pledge_delta_total += &result.pledge_delta;

        state
            .apply_penalty(&penalty_target)
            .map_err(|e| actor_error_v14!(illegal_state, "failed to apply penalty: {}", e))?;

        log::debug!(
            "storage provider {} penalized {} for continued fault",
            rt.message().receiver(),
            penalty_target
        );

        let (penalty_from_vesting, penalty_from_balance) = state
            .repay_partial_debt_in_priority_order(
                rt.store(),
                rt.curr_epoch(),
                &rt.current_balance(),
            )
            .map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to unlock penalty")
            })?;

        penalty_total = &penalty_from_vesting + penalty_from_balance;
        pledge_delta_total -= penalty_from_vesting;

        continue_cron = state.continue_deadline_cron();
        if !continue_cron {
            state.deadline_cron_active = false;
        }

        Ok(state.clone())
    })?;

    // Remove power for new faults, and burn penalties.
    request_update_power(rt, power_delta_total)?;
    burn_funds(rt, penalty_total)?;
    notify_pledge_changed(rt, &pledge_delta_total)?;

    // Schedule cron callback for next deadline's last epoch.
    if continue_cron {
        let new_deadline_info = state.deadline_info(rt.policy(), curr_epoch + 1);
        enroll_cron_event(
            rt,
            new_deadline_info.last(),
            CronEventPayload { event_type: CRON_EVENT_PROVING_DEADLINE },
        )?;
    } else {
        info!("miner {} going inactive, deadline cron discontinued", rt.message().receiver())
    }

    // Record whether or not we _have_ early terminations now.
    let has_early_terminations = have_pending_early_terminations(&state);

    // If we didn't have pending early terminations before, but we do now,
    // handle them at the next epoch.
    if !had_early_terminations && has_early_terminations {
        // First, try to process some of these terminations.
        if process_early_terminations(rt, reward_smoothed, quality_adj_power_smoothed)? {
            // If that doesn't work, just defer till the next epoch.
            schedule_early_termination_work(rt)?;
        }

        // Note: _don't_ process early terminations if we had a cron
        // callback already scheduled. In that case, we'll already have
        // processed AddressedSectorsMax terminations this epoch.
    }

    Ok(())
}

fn validate_expiration(
    policy: &Policy,
    curr_epoch: ChainEpoch,
    activation: ChainEpoch,
    expiration: ChainEpoch,
    seal_proof: RegisteredSealProof,
) -> Result<(), ActorError> {
    // Expiration must be after activation. Check this explicitly to avoid an underflow below.
    if expiration <= activation {
        return Err(actor_error_v14!(
            illegal_argument,
            "sector expiration {} must be after activation {}",
            expiration,
            activation
        ));
    }

    // expiration cannot be less than minimum after activation
    if expiration - activation < policy.min_sector_expiration {
        return Err(actor_error_v14!(
            illegal_argument,
            "invalid expiration {}, total sector lifetime ({}) must exceed {} after activation {}",
            expiration,
            expiration - activation,
            policy.min_sector_expiration,
            activation
        ));
    }

    // expiration cannot exceed MaxSectorExpirationExtension from now
    if expiration > curr_epoch + policy.max_sector_expiration_extension {
        return Err(actor_error_v14!(
            illegal_argument,
            "invalid expiration {}, cannot be more than {} past current epoch {}",
            expiration,
            policy.max_sector_expiration_extension,
            curr_epoch
        ));
    }

    // total sector lifetime cannot exceed SectorMaximumLifetime for the sector's seal proof
    let max_lifetime = seal_proof_sector_maximum_lifetime(seal_proof).ok_or_else(|| {
        actor_error_v14!(illegal_argument, "unrecognized seal proof type {:?}", seal_proof)
    })?;
    if expiration - activation > max_lifetime {
        return Err(actor_error_v14!(
        illegal_argument,
        "invalid expiration {}, total sector lifetime ({}) cannot exceed {} after activation {}",
        expiration,
        expiration - activation,
        max_lifetime,
        activation
    ));
    }

    Ok(())
}

fn enroll_cron_event(
    rt: &impl Runtime,
    event_epoch: ChainEpoch,
    cb: CronEventPayload,
) -> Result<(), ActorError> {
    let payload = serialize(&cb, "cron payload")?;
    let ser_params =
        IpldBlock::serialize_cbor(&ext::power::EnrollCronEventParams { event_epoch, payload })?;
    extract_send_result(rt.send_simple(
        &STORAGE_POWER_ACTOR_ADDR,
        ext::power::ENROLL_CRON_EVENT_METHOD,
        ser_params,
        TokenAmount::zero(),
    ))?;

    Ok(())
}

fn request_update_power(rt: &impl Runtime, delta: PowerPair) -> Result<(), ActorError> {
    if delta.is_zero() {
        return Ok(());
    }

    let delta_clone = delta.clone();

    extract_send_result(rt.send_simple(
        &STORAGE_POWER_ACTOR_ADDR,
        ext::power::UPDATE_CLAIMED_POWER_METHOD,
        IpldBlock::serialize_cbor(&ext::power::UpdateClaimedPowerParams {
            raw_byte_delta: delta.raw,
            quality_adjusted_delta: delta.qa,
        })?,
        TokenAmount::zero(),
    ))
    .map_err(|e| e.wrap(format!("failed to update power with {:?}", delta_clone)))?;

    Ok(())
}

fn request_terminate_deals(
    rt: &impl Runtime,
    epoch: ChainEpoch,
    sectors: &BitField,
) -> Result<(), ActorError> {
    if !sectors.is_empty() {
        // The sectors bitfield could be large, but will fit into a single parameters block.
        // The FVM max block size of 1MiB supports 130K 8-byte integers, but the policy parameter
        // ADDRESSED_SECTORS_MAX (currently 25k) will avoid reaching that.
        let res = extract_send_result(rt.send_simple(
            &STORAGE_MARKET_ACTOR_ADDR,
            ext::market::ON_MINER_SECTORS_TERMINATE_METHOD,
            IpldBlock::serialize_cbor(&ext::market::OnMinerSectorsTerminateParams {
                epoch,
                sectors: sectors.clone(),
            })?,
            TokenAmount::zero(),
        ));
        // If running in a system / cron context intentionally swallow this error to prevent
        // frozen market cron corruption from also freezing this miner cron.
        if rt.message().origin() == SYSTEM_ACTOR_ADDR {
            if let Err(e) = res {
                error!("OnSectorsTerminate event failed from cron caller {}", e)
            }
        } else {
            res?;
        }
    }
    Ok(())
}

fn schedule_early_termination_work(rt: &impl Runtime) -> Result<(), ActorError> {
    info!("scheduling early terminations with cron...");
    enroll_cron_event(
        rt,
        rt.curr_epoch() + 1,
        CronEventPayload { event_type: CRON_EVENT_PROCESS_EARLY_TERMINATIONS },
    )
}

fn have_pending_early_terminations(state: &State) -> bool {
    let no_early_terminations = state.early_terminations.is_empty();
    !no_early_terminations
}

// returns true if valid, false if invalid, error if failed to validate either way!
fn verify_windowed_post(
    rt: &impl Runtime,
    challenge_epoch: ChainEpoch,
    sectors: &[SectorOnChainInfo],
    proofs: Vec<PoStProof>,
) -> Result<bool, ActorError> {
    let miner_actor_id: u64 = if let Payload::ID(i) = rt.message().receiver().payload() {
        *i
    } else {
        return Err(actor_error_v14!(
            illegal_state,
            "runtime provided bad receiver address {}",
            rt.message().receiver()
        ));
    };

    // Regenerate challenge randomness, which must match that generated for the proof.
    let entropy = serialize(&rt.message().receiver(), "address for window post challenge")?;
    let randomness = rt.get_randomness_from_beacon(
        DomainSeparationTag::WindowedPoStChallengeSeed,
        challenge_epoch,
        &entropy,
    )?;

    let challenged_sectors = sectors
        .iter()
        .map(|s| SectorInfo {
            proof: s.seal_proof,
            sector_number: s.sector_number,
            sealed_cid: s.sealed_cid,
        })
        .collect();

    // get public inputs
    let pv_info = WindowPoStVerifyInfo {
        randomness: Randomness(randomness.into()),
        proofs,
        challenged_sectors,
        prover: miner_actor_id,
    };

    // verify the post proof
    let result = rt.verify_post(&pv_info);
    Ok(result.is_ok())
}

struct SectorSealProofInput {
    pub registered_proof: RegisteredSealProof,
    pub sector_number: SectorNumber,
    pub randomness: SealRandomness,
    pub interactive_randomness: InteractiveSealRandomness,
    // Commr
    pub sealed_cid: Cid,
    // Commd
    pub unsealed_cid: Cid,
}

impl SectorSealProofInput {
    fn to_seal_verify_info(&self, miner_actor_id: u64, proof: &RawBytes) -> SealVerifyInfo {
        SealVerifyInfo {
            registered_proof: self.registered_proof,
            sector_id: SectorID { miner: miner_actor_id, number: self.sector_number },
            deal_ids: vec![], // unused by the proofs api so this is safe to leave empty
            randomness: self.randomness.clone(),
            interactive_randomness: self.interactive_randomness.clone(),
            proof: proof.clone().into(),
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

// Validates pre-committed sectors are ready for proving and committing this epoch.
// Returns seal proof verification inputs for every pre-commit, even those that fail validation.
// The proof verification inputs are needed as witnesses to verify an aggregated proof to allow
// other, valid, sectors to succeed.
fn validate_precommits(
    rt: &impl Runtime,
    precommits: &[SectorPreCommitOnChainInfo],
    allow_deal_ids: bool,
    all_or_nothing: bool,
) -> Result<(BatchReturn, Vec<SectorSealProofInput>), ActorError> {
    if precommits.is_empty() {
        return Ok((BatchReturn::empty(), vec![]));
    }
    let mut batch = BatchReturnGen::new(precommits.len());

    let mut verify_infos = vec![];
    for (i, precommit) in precommits.iter().enumerate() {
        // We record failures and continue validation rather than continuing the loop in order to:
        // 1. compute aggregate seal verification inputs
        // 2. check for whole message failure conditions
        let mut fail_validation = false;
        if !(allow_deal_ids || precommit.info.deal_ids.is_empty()) {
            warn!(
                "skipping commitment for sector {}, precommit has deal ids which are disallowed",
                precommit.info.sector_number,
            );
            fail_validation = true;
        }
        let msd =
            max_prove_commit_duration(rt.policy(), precommit.info.seal_proof).ok_or_else(|| {
                actor_error_v14!(
                    illegal_state,
                    "no max seal duration for proof type: {}",
                    i64::from(precommit.info.seal_proof)
                )
            })?;
        let prove_commit_due = precommit.pre_commit_epoch + msd;
        if rt.curr_epoch() > prove_commit_due {
            warn!(
                "skipping commitment for sector {}, too late at {}, due {}",
                precommit.info.sector_number,
                rt.curr_epoch(),
                prove_commit_due,
            );
            fail_validation = true
        }

        // All seal proof types should match
        if i >= 1 {
            let prev_seal_proof = precommits[i - 1].info.seal_proof;
            if prev_seal_proof != precommit.info.seal_proof {
                return Err(actor_error_v14!(
                    illegal_state,
                    "seal proof group for verification contains mismatched seal proofs {} and {}",
                    i64::from(prev_seal_proof),
                    i64::from(precommit.info.seal_proof)
                ));
            }
        }
        let interactive_epoch = precommit.pre_commit_epoch + rt.policy().pre_commit_challenge_delay;
        if rt.curr_epoch() <= interactive_epoch {
            return Err(actor_error_v14!(forbidden, "too early to prove sector"));
        }

        // Compute svi for all commits even those that will not be activated.
        // Callers might prove using aggregates and need witnesses for invalid commits.
        let entropy = serialize(&rt.message().receiver(), "address for get verify info")?;
        let randomness = Randomness(
            rt.get_randomness_from_tickets(
                DomainSeparationTag::SealRandomness,
                precommit.info.seal_rand_epoch,
                &entropy,
            )?
            .into(),
        );
        let interactive_randomness = Randomness(
            rt.get_randomness_from_beacon(
                DomainSeparationTag::InteractiveSealChallengeSeed,
                interactive_epoch,
                &entropy,
            )?
            .into(),
        );

        let unsealed_cid = precommit.info.unsealed_cid.get_cid(precommit.info.seal_proof)?;
        verify_infos.push(SectorSealProofInput {
            registered_proof: precommit.info.seal_proof,
            sector_number: precommit.info.sector_number,
            randomness,
            interactive_randomness,
            sealed_cid: precommit.info.sealed_cid,
            unsealed_cid,
        });

        if fail_validation {
            if all_or_nothing {
                return Err(actor_error_v14!(
                    illegal_argument,
                    "invalid pre-commit {} while requiring activation success: {:?}",
                    i,
                    precommit
                ));
            }
            batch.add_fail(ExitCode::USR_ILLEGAL_ARGUMENT);
        } else {
            batch.add_success();
        }
    }
    Ok((batch.gen(), verify_infos))
}

fn validate_ni_sectors(
    rt: &impl Runtime,
    sectors: &[SectorNIActivationInfo],
    seal_proof_type: RegisteredSealProof,
    all_or_nothing: bool,
) -> Result<(BatchReturn, Vec<SectorSealProofInput>), ActorError> {
    let receiver = rt.message().receiver();
    let miner_id = receiver.id().unwrap();
    let curr_epoch = rt.curr_epoch();
    let activation_epoch = curr_epoch;
    let challenge_earliest = curr_epoch - rt.policy().max_prove_commit_ni_randomness_lookback;
    let unsealed_cid = CompactCommD::empty().get_cid(seal_proof_type).unwrap();
    let entropy = serialize(&receiver, "address for get verify info")?;

    if sectors.is_empty() {
        return Ok((BatchReturn::empty(), vec![]));
    }
    let mut batch = BatchReturnGen::new(sectors.len());

    let mut verify_infos = vec![];
    let mut sector_numbers = BitField::new();
    for (i, sector) in sectors.iter().enumerate() {
        let mut fail_validation = false;

        let set = sector_numbers.get(sector.sector_number);
        if set {
            warn!("duplicate sector number {}", sector.sector_number);
            fail_validation = true;
        }

        if sector.sector_number > MAX_SECTOR_NUMBER {
            warn!("sector number {} out of range 0..(2^63-1)", sector.sector_number);
            fail_validation = true;
        }

        sector_numbers.set(sector.sector_number);

        if let Err(err) = validate_expiration(
            rt.policy(),
            curr_epoch,
            activation_epoch,
            sector.expiration,
            seal_proof_type,
        ) {
            warn!("invalid expiration: {}", err);
            fail_validation = true;
        }

        if sector.sealer_id != miner_id {
            warn!("sealer must be the same as the receiver actor for all sectors");
            fail_validation = true;
        }

        if sector.sector_number != sector.sealing_number {
            warn!("sealing number must be same as sector number for all sectors");
            fail_validation = true;
        }

        if !is_sealed_sector(&sector.sealed_cid) {
            warn!("sealed CID had wrong prefix");
            fail_validation = true;
        }

        if sector.seal_rand_epoch >= curr_epoch {
            warn!(
                "seal challenge epoch {} must be before now {}",
                sector.seal_rand_epoch, curr_epoch
            );
            fail_validation = true;
        }

        if sector.seal_rand_epoch < challenge_earliest {
            warn!(
                "seal challenge epoch {} too old, must be after {}",
                sector.seal_rand_epoch, challenge_earliest
            );
            fail_validation = true;
        }

        verify_infos.push(SectorSealProofInput {
            registered_proof: seal_proof_type,
            sector_number: sector.sealing_number,
            randomness: Randomness(
                rt.get_randomness_from_tickets(
                    DomainSeparationTag::SealRandomness,
                    sector.seal_rand_epoch,
                    &entropy,
                )?
                .into(),
            ),
            interactive_randomness: Randomness(vec![1u8; 32]),
            sealed_cid: sector.sealed_cid,
            unsealed_cid,
        });

        if fail_validation {
            if all_or_nothing {
                return Err(actor_error_v14!(
                    illegal_argument,
                    "invalid NI commit {} while requiring activation success: {:?}",
                    i,
                    sector
                ));
            }
            batch.add_fail(ExitCode::USR_ILLEGAL_ARGUMENT);
        } else {
            batch.add_success();
        }
    }

    Ok((batch.gen(), verify_infos))
}

// Validates a batch of sector sealing proofs.
fn validate_seal_proofs(
    seal_proof_type: RegisteredSealProof,
    proofs: &[RawBytes],
) -> Result<(), ActorError> {
    let max_proof_size =
        seal_proof_type.proof_size().with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
            format!("failed to determine max proof size for type {:?}", seal_proof_type,)
        })?;
    for proof in proofs {
        if proof.len() > max_proof_size {
            return Err(actor_error_v14!(
                illegal_argument,
                "sector proof size {} exceeds max {}",
                proof.len(),
                max_proof_size
            ));
        }
    }
    Ok(())
}

fn validate_seal_aggregate_proof(
    proof: &RawBytes,
    sector_count: u64,
    policy: &Policy,
    interactive: bool,
) -> Result<(), ActorError> {
    let (min, max) = match interactive {
        false => (policy.min_aggregated_sectors, policy.max_aggregated_sectors),
        true => (policy.min_aggregated_sectors_ni, policy.max_aggregated_sectors_ni),
    };

    if sector_count > max {
        return Err(actor_error_v14!(
            illegal_argument,
            "too many sectors addressed, addressed {} want <= {}",
            sector_count,
            max
        ));
    } else if sector_count < min {
        return Err(actor_error_v14!(
            illegal_argument,
            "too few sectors addressed, addressed {} want >= {}",
            sector_count,
            min
        ));
    }
    if proof.len() > policy.max_aggregated_proof_size {
        return Err(actor_error_v14!(
            illegal_argument,
            "sector prove-commit proof of size {} exceeds max size of {}",
            proof.len(),
            policy.max_aggregated_proof_size
        ));
    }
    Ok(())
}

fn verify_aggregate_seal(
    rt: &impl Runtime,
    proof_inputs: &[SectorSealProofInput],
    miner_actor_id: ActorID,
    seal_proof: RegisteredSealProof,
    aggregate_proof: RegisteredAggregateProof,
    proof_bytes: &RawBytes,
) -> Result<(), ActorError> {
    let seal_verify_inputs =
        proof_inputs.iter().map(|pi| pi.to_aggregate_seal_verify_info()).collect();

    rt.verify_aggregate_seals(&AggregateSealVerifyProofAndInfos {
        miner: miner_actor_id,
        seal_proof,
        aggregate_proof,
        proof: proof_bytes.clone().into(),
        infos: seal_verify_inputs,
    })
    .context_code(ExitCode::USR_ILLEGAL_ARGUMENT, "aggregate seal verify failed")
}

// Compute and burn the aggregate network fee.
fn pay_aggregate_seal_proof_fee(
    rt: &impl Runtime,
    aggregate_size: usize,
) -> Result<(), ActorError> {
    // State is loaded afresh as earlier operations for sector/data activation can change it.
    let state: State = rt.state()?;
    let aggregate_fee = aggregate_prove_commit_network_fee(aggregate_size, &rt.base_fee());
    let unlocked_balance = state
        .get_unlocked_balance(&rt.current_balance())
        .map_err(|_e| actor_error_v14!(illegal_state, "failed to determine unlocked balance"))?;
    if unlocked_balance < aggregate_fee {
        return Err(actor_error_v14!(
                insufficient_funds,
                "remaining unlocked funds after prove-commit {} are insufficient to pay aggregation fee of {}",
                unlocked_balance,
                aggregate_fee
            ));
    }
    burn_funds(rt, aggregate_fee)?;
    state.check_balance_invariants(&rt.current_balance()).map_err(balance_invariants_broken)
}

fn verify_deals(
    rt: &impl Runtime,
    sectors: &[ext::market::SectorDeals],
) -> Result<ext::market::VerifyDealsForActivationReturn, ActorError> {
    // Short-circuit if there are no deals in any of the sectors.
    let mut deal_count = 0;
    for sector in sectors {
        deal_count += sector.deal_ids.len();
    }
    if deal_count == 0 {
        return Ok(ext::market::VerifyDealsForActivationReturn {
            unsealed_cids: vec![None; sectors.len()],
        });
    }

    deserialize_block(extract_send_result(rt.send_simple(
        &STORAGE_MARKET_ACTOR_ADDR,
        ext::market::VERIFY_DEALS_FOR_ACTIVATION_METHOD,
        IpldBlock::serialize_cbor(&ext::market::VerifyDealsForActivationParamsRef { sectors })?,
        TokenAmount::zero(),
    ))?)
}

/// Requests the current epoch target block reward from the reward actor.
/// return value includes reward, smoothed estimate of reward, and baseline power
fn request_current_epoch_block_reward(
    rt: &impl Runtime,
) -> Result<ThisEpochRewardReturn, ActorError> {
    deserialize_block(
        extract_send_result(rt.send_simple(
            &REWARD_ACTOR_ADDR,
            ext::reward::THIS_EPOCH_REWARD_METHOD,
            Default::default(),
            TokenAmount::zero(),
        ))
        .map_err(|e| e.wrap("failed to check epoch baseline power"))?,
    )
}

/// Requests the current network total power and pledge from the power actor.
fn request_current_total_power(
    rt: &impl Runtime,
) -> Result<ext::power::CurrentTotalPowerReturn, ActorError> {
    deserialize_block(
        extract_send_result(rt.send_simple(
            &STORAGE_POWER_ACTOR_ADDR,
            ext::power::CURRENT_TOTAL_POWER_METHOD,
            Default::default(),
            TokenAmount::zero(),
        ))
        .map_err(|e| e.wrap("failed to check current power"))?,
    )
}

/// Resolves an address to an ID address and verifies that it is address of an account actor with an associated BLS key.
/// The worker must be BLS since the worker key will be used alongside a BLS-VRF.
fn resolve_worker_address(rt: &impl Runtime, raw: Address) -> Result<ActorID, ActorError> {
    let resolved = rt
        .resolve_address(&raw)
        .ok_or_else(|| actor_error_v14!(illegal_argument, "unable to resolve address: {}", raw))?;

    let worker_code = rt
        .get_actor_code_cid(&resolved)
        .ok_or_else(|| actor_error_v14!(illegal_argument, "no code for address: {}", resolved))?;
    if rt.resolve_builtin_actor_type(&worker_code) != Some(Type::Account) {
        return Err(actor_error_v14!(
            illegal_argument,
            "worker actor type must be an account, was {}",
            worker_code
        ));
    }

    if raw.protocol() != Protocol::BLS {
        let pub_key: Address = deserialize_block(extract_send_result(rt.send_simple(
            &Address::new_id(resolved),
            ext::account::PUBKEY_ADDRESS_METHOD,
            None,
            TokenAmount::zero(),
        ))?)?;
        if pub_key.protocol() != Protocol::BLS {
            return Err(actor_error_v14!(
                illegal_argument,
                "worker account {} must have BLS pubkey, was {}",
                resolved,
                pub_key.protocol()
            ));
        }
    }
    Ok(resolved)
}

fn burn_funds(rt: &impl Runtime, amount: TokenAmount) -> Result<(), ActorError> {
    log::debug!("storage provder {} burning {}", rt.message().receiver(), amount);
    if amount.is_positive() {
        extract_send_result(rt.send_simple(&BURNT_FUNDS_ACTOR_ADDR, METHOD_SEND, None, amount))?;
    }
    Ok(())
}

fn notify_pledge_changed(rt: &impl Runtime, pledge_delta: &TokenAmount) -> Result<(), ActorError> {
    if !pledge_delta.is_zero() {
        extract_send_result(rt.send_simple(
            &STORAGE_POWER_ACTOR_ADDR,
            ext::power::UPDATE_PLEDGE_TOTAL_METHOD,
            IpldBlock::serialize_cbor(pledge_delta)?,
            TokenAmount::zero(),
        ))?;
    }
    Ok(())
}

fn get_claims(
    rt: &impl Runtime,
    ids: &[ext::verifreg::ClaimID],
) -> Result<Vec<ext::verifreg::Claim>, ActorError> {
    let params = ext::verifreg::GetClaimsParams {
        provider: rt.message().receiver().id().unwrap(),
        claim_ids: ids.to_owned(),
    };
    let claims_ret: ext::verifreg::GetClaimsReturn =
        deserialize_block(extract_send_result(rt.send_simple(
            &VERIFIED_REGISTRY_ACTOR_ADDR,
            ext::verifreg::GET_CLAIMS_METHOD,
            IpldBlock::serialize_cbor(&params)?,
            TokenAmount::zero(),
        ))?)?;
    if (claims_ret.batch_info.success_count as usize) < ids.len() {
        return Err(actor_error_v14!(illegal_argument, "invalid claims"));
    }
    Ok(claims_ret.claims)
}

/// Assigns proving period offset randomly in the range [0, WPoStProvingPeriod) by hashing
/// the actor's address and current epoch.
fn assign_proving_period_offset(
    policy: &Policy,
    addr: Address,
    current_epoch: ChainEpoch,
    blake2b: impl FnOnce(&[u8]) -> [u8; 32],
) -> anyhow::Result<ChainEpoch> {
    let mut my_addr = serialize_vec(&addr, "address")?;
    my_addr.write_i64::<BigEndian>(current_epoch)?;

    let digest = blake2b(&my_addr);

    let mut offset: u64 = BigEndian::read_u64(&digest);
    offset %= policy.wpost_proving_period as u64;

    // Conversion from i64 to u64 is safe because it's % WPOST_PROVING_PERIOD which is i64
    Ok(offset as ChainEpoch)
}

/// Computes the epoch at which a proving period should start such that it is greater than the current epoch, and
/// has a defined offset from being an exact multiple of WPoStProvingPeriod.
/// A miner is exempt from Winow PoSt until the first full proving period starts.
fn current_proving_period_start(
    policy: &Policy,
    current_epoch: ChainEpoch,
    offset: ChainEpoch,
) -> ChainEpoch {
    let curr_modulus = current_epoch % policy.wpost_proving_period;

    let period_progress = if curr_modulus >= offset {
        curr_modulus - offset
    } else {
        policy.wpost_proving_period - (offset - curr_modulus)
    };

    current_epoch - period_progress
}

fn current_deadline_index(
    policy: &Policy,
    current_epoch: ChainEpoch,
    period_start: ChainEpoch,
) -> u64 {
    ((current_epoch - period_start) / policy.wpost_challenge_window) as u64
}

/// Computes deadline information for a fault or recovery declaration.
/// If the deadline has not yet elapsed, the declaration is taken as being for the current proving period.
/// If the deadline has elapsed, it's instead taken as being for the next proving period after the current epoch.
fn declaration_deadline_info(
    policy: &Policy,
    period_start: ChainEpoch,
    deadline_idx: u64,
    current_epoch: ChainEpoch,
) -> anyhow::Result<DeadlineInfo> {
    if deadline_idx >= policy.wpost_period_deadlines {
        return Err(anyhow!(
            "invalid deadline {}, must be < {}",
            deadline_idx,
            policy.wpost_period_deadlines
        ));
    }

    let deadline =
        new_deadline_info(policy, period_start, deadline_idx, current_epoch).next_not_elapsed();
    Ok(deadline)
}

/// Checks that a fault or recovery declaration at a specific deadline is outside the exclusion window for the deadline.
fn validate_fr_declaration_deadline(deadline: &DeadlineInfo) -> anyhow::Result<()> {
    if deadline.fault_cutoff_passed() {
        Err(anyhow!("late fault or recovery declaration"))
    } else {
        Ok(())
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

fn consensus_fault_active(info: &MinerInfo, curr_epoch: ChainEpoch) -> bool {
    // For penalization period to last for exactly finality epochs
    // consensus faults are active until currEpoch exceeds ConsensusFaultElapsed
    curr_epoch <= info.consensus_fault_elapsed
}

pub fn power_for_sector(sector_size: SectorSize, sector: &SectorOnChainInfo) -> PowerPair {
    PowerPair {
        raw: BigInt::from(sector_size as u64),
        qa: qa_power_for_sector(sector_size, sector),
    }
}

/// Returns the sum of the raw byte and quality-adjusted power for sectors.
pub fn power_for_sectors(sector_size: SectorSize, sectors: &[SectorOnChainInfo]) -> PowerPair {
    let qa = sectors.iter().map(|s| qa_power_for_sector(sector_size, s)).sum();

    PowerPair { raw: BigInt::from(sector_size as u64) * BigInt::from(sectors.len()), qa }
}

fn get_miner_info<BS>(store: &BS, state: &State) -> Result<MinerInfo, ActorError>
where
    BS: Blockstore,
{
    state
        .get_info(store)
        .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "could not read miner info"))
}

fn process_pending_worker(
    info: &mut MinerInfo,
    rt: &impl Runtime,
    state: &mut State,
) -> Result<(), ActorError> {
    let pending_worker_key = if let Some(k) = &info.pending_worker_key {
        k
    } else {
        return Ok(());
    };

    if rt.curr_epoch() < pending_worker_key.effective_at {
        return Ok(());
    }

    info.worker = pending_worker_key.new_worker;
    info.pending_worker_key = None;

    state
        .save_info(rt.store(), info)
        .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save miner info"))
}

/// Repays all fee debt and then verifies that the miner has amount needed to cover
/// the pledge requirement after burning all fee debt.  If not aborts.
/// Returns an amount that must be burnt by the actor.
/// Note that this call does not compute recent vesting so reported unlocked balance
/// may be slightly lower than the true amount. Computing vesting here would be
/// almost always redundant since vesting is quantized to ~daily units.  Vesting
/// will be at most one proving period old if computed in the cron callback.
fn repay_debts_or_abort(rt: &impl Runtime, state: &mut State) -> Result<TokenAmount, ActorError> {
    let res = state.repay_debts(&rt.current_balance()).map_err(|e| {
        e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "unlocked balance can not repay fee debt")
    })?;
    info!("RepayDebtsOrAbort was called and succeeded");
    Ok(res)
}

fn check_control_addresses(policy: &Policy, control_addrs: &[Address]) -> Result<(), ActorError> {
    if control_addrs.len() > policy.max_control_addresses {
        return Err(actor_error_v14!(
            illegal_argument,
            "control addresses length {} exceeds max control addresses length {}",
            control_addrs.len(),
            policy.max_control_addresses
        ));
    }

    Ok(())
}

fn check_valid_post_proof_type(
    policy: &Policy,
    proof_type: RegisteredPoStProof,
) -> Result<(), ActorError> {
    if policy.valid_post_proof_type.contains(proof_type) {
        Ok(())
    } else {
        Err(actor_error_v14!(
            illegal_argument,
            "proof type {:?} not allowed for new miner actors",
            proof_type
        ))
    }
}

fn check_peer_info(
    policy: &Policy,
    peer_id: &[u8],
    multiaddrs: &[BytesDe],
) -> Result<(), ActorError> {
    if peer_id.len() > policy.max_peer_id_length {
        return Err(actor_error_v14!(
            illegal_argument,
            "peer ID size of {} exceeds maximum size of {}",
            peer_id.len(),
            policy.max_peer_id_length
        ));
    }

    let mut total_size = 0;
    for ma in multiaddrs {
        if ma.0.is_empty() {
            return Err(actor_error_v14!(illegal_argument, "invalid empty multiaddr"));
        }
        total_size += ma.0.len();
    }

    if total_size > policy.max_multiaddr_data {
        return Err(actor_error_v14!(
            illegal_argument,
            "multiaddr size of {} exceeds maximum of {}",
            total_size,
            policy.max_multiaddr_data
        ));
    }

    Ok(())
}

fn activate_new_sector_infos(
    rt: &impl Runtime,
    precommits: Vec<&SectorPreCommitOnChainInfo>,
    data_activations: Vec<DataActivationOutput>,
    pledge_inputs: &NetworkPledgeInputs,
    info: &MinerInfo,
) -> Result<(), ActorError> {
    let activation_epoch = rt.curr_epoch();

    let (total_pledge, newly_vested) = rt.transaction(|state: &mut State, rt| {
        let policy = rt.policy();
        let store = rt.store();

        let mut new_sector_numbers = Vec::<SectorNumber>::with_capacity(data_activations.len());
        let mut deposit_to_unlock = TokenAmount::zero();
        let mut new_sectors = Vec::<SectorOnChainInfo>::new();
        let mut total_pledge = TokenAmount::zero();

        for (pci, deal_spaces) in precommits.iter().zip(data_activations) {
            // compute initial pledge
            let duration = pci.info.expiration - activation_epoch;
            // This is probably always caught in precommit but fail cleanly if it occurs
            if duration < policy.min_sector_expiration {
                return Err(actor_error_v14!(
                    illegal_argument,
                    "precommit {} has lifetime {} less than minimum {}. ignoring",
                    pci.info.sector_number,
                    duration,
                    policy.min_sector_expiration
                ));
            }

            let deal_weight = &deal_spaces.unverified_space * duration;
            let verified_deal_weight = &deal_spaces.verified_space * duration;

            let power = qa_power_for_weight(
                info.sector_size,
                duration,
                &deal_weight,
                &verified_deal_weight,
            );

            let day_reward = expected_reward_for_power(
                &pledge_inputs.epoch_reward,
                &pledge_inputs.network_qap,
                &power,
                fil_actors_shared::v14::EPOCHS_IN_DAY,
            );

            // The storage pledge is recorded for use in computing the penalty if this sector is terminated
            // before its declared expiration.
            // It's not capped to 1 FIL, so can exceed the actual initial pledge requirement.
            let storage_pledge = expected_reward_for_power(
                &pledge_inputs.epoch_reward,
                &pledge_inputs.network_qap,
                &power,
                INITIAL_PLEDGE_PROJECTION_PERIOD,
            );

            let initial_pledge = initial_pledge_for_power(
                &power,
                &pledge_inputs.network_baseline,
                &pledge_inputs.epoch_reward,
                &pledge_inputs.network_qap,
                &pledge_inputs.circulating_supply,
            );

            deposit_to_unlock += pci.pre_commit_deposit.clone();
            total_pledge += &initial_pledge;

            let new_sector_info = SectorOnChainInfo {
                sector_number: pci.info.sector_number,
                seal_proof: pci.info.seal_proof,
                sealed_cid: pci.info.sealed_cid,
                deprecated_deal_ids: vec![], // deal ids field deprecated
                expiration: pci.info.expiration,
                activation: activation_epoch,
                deal_weight,
                verified_deal_weight,
                initial_pledge,
                expected_day_reward: day_reward,
                expected_storage_pledge: storage_pledge,
                power_base_epoch: activation_epoch,
                replaced_day_reward: TokenAmount::zero(),
                sector_key_cid: None,
                flags: SectorOnChainInfoFlags::SIMPLE_QA_POWER,
            };

            new_sector_numbers.push(new_sector_info.sector_number);
            new_sectors.push(new_sector_info);
        }

        state.put_sectors(store, new_sectors.clone()).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to put new sectors")
        })?;
        state.delete_precommitted_sectors(store, &new_sector_numbers)?;
        state
            .assign_sectors_to_deadlines(
                policy,
                store,
                rt.curr_epoch(),
                new_sectors,
                info.window_post_partition_sectors,
                info.sector_size,
            )
            .map_err(|e| {
                e.downcast_default(
                    ExitCode::USR_ILLEGAL_STATE,
                    "failed to assign new sectors to deadlines",
                )
            })?;
        let newly_vested = TokenAmount::zero();

        // Unlock deposit for successful proofs, make it available for lock-up as initial pledge.
        state
            .add_pre_commit_deposit(&(-deposit_to_unlock))
            .map_err(|e| actor_error_v14!(illegal_state, "failed to add precommit deposit: {}", e))?;

        let unlocked_balance = state.get_unlocked_balance(&rt.current_balance()).map_err(|e| {
            actor_error_v14!(illegal_state, "failed to calculate unlocked balance: {}", e)
        })?;
        if unlocked_balance < total_pledge {
            return Err(actor_error_v14!(
                insufficient_funds,
                "insufficient funds for aggregate initial pledge requirement {}, available: {}",
                total_pledge,
                unlocked_balance
            ));
        }

        state
            .add_initial_pledge(&total_pledge)
            .map_err(|e| actor_error_v14!(illegal_state, "failed to add initial pledge: {}", e))?;

        state.check_balance_invariants(&rt.current_balance()).map_err(balance_invariants_broken)?;

        Ok((total_pledge, newly_vested))
    })?;
    // Request pledge update for activated sectors.
    // Power is not activated until first Window poST.
    notify_pledge_changed(rt, &(total_pledge - newly_vested))?;

    Ok(())
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

// Data activation results for one sector
#[derive(Clone)]
struct DataActivationOutput {
    pub unverified_space: BigInt,
    pub verified_space: BigInt,
    // None indicates either no deals or computation was not requested.
    pub unsealed_cid: Option<Cid>,
    pub pieces: Vec<(Cid, u64)>,
}

// Track information needed to update a sector info's data during ProveReplicaUpdate
#[derive(Clone, Debug)]
struct UpdateAndSectorInfo<'a> {
    update: &'a ReplicaUpdateInner,
    sector_info: &'a SectorOnChainInfo,
}

// Inputs to state update for a single sector replica update.
struct ReplicaUpdateStateInputs<'a> {
    deadline: u64,
    partition: u64,
    sector_info: &'a SectorOnChainInfo,
    activated_data: ReplicaUpdateActivatedData,
}

// Summary of activated data for a replica update.
struct ReplicaUpdateActivatedData {
    seal_cid: Cid,
    unverified_space: BigInt,
    verified_space: BigInt,
}

// Activates data pieces by claiming allocations with the verified registry.
// Pieces are grouped by sector and succeed or fail in sector groups.
// If an activation input specifies an expected CommD for the sector, a CommD
// is calculated from the pieces and must match.
// This method never returns CommDs in the output type; either the caller provided
// them and they are correct, or the caller did not provide anything that needs checking.
fn activate_sectors_pieces(
    rt: &impl Runtime,
    activation_inputs: Vec<SectorPiecesActivationInput>,
    all_or_nothing: bool,
) -> Result<(BatchReturn, Vec<DataActivationOutput>), ActorError> {
    // Get a flattened list of verified claims for all activated sectors
    let mut verified_claims = Vec::new();
    let mut sectors_pieces = Vec::new();

    for activation_info in &activation_inputs {
        // Check a declared CommD matches that computed from the data.
        if let Some(declared_commd) = &activation_info.expected_commd {
            let computed_commd = unsealed_cid_from_pieces(
                rt,
                &activation_info.piece_manifests,
                activation_info.sector_type,
            )?
            .get_cid(activation_info.sector_type)?;
            // A declared zero CommD might be compact or fully computed,
            // so normalize to the computed value before checking.
            if !declared_commd.get_cid(activation_info.sector_type)?.eq(&computed_commd) {
                return Err(actor_error_v14!(
                    illegal_argument,
                    "unsealed CID does not match pieces for sector {}, computed {:?} declared {:?}",
                    activation_info.sector_number,
                    computed_commd,
                    declared_commd
                ));
            }
        }

        let mut sector_claims = vec![];
        sectors_pieces.push(&activation_info.piece_manifests);

        for piece in &activation_info.piece_manifests {
            if let Some(alloc_key) = &piece.verified_allocation_key {
                sector_claims.push(ext::verifreg::AllocationClaim {
                    client: alloc_key.client,
                    allocation_id: alloc_key.id,
                    data: piece.cid,
                    size: piece.size,
                });
            }
        }
        verified_claims.push(ext::verifreg::SectorAllocationClaims {
            sector: activation_info.sector_number,
            expiry: activation_info.sector_expiry,
            claims: sector_claims,
        });
    }
    let claim_res = batch_claim_allocations(rt, verified_claims, all_or_nothing)?;
    if all_or_nothing {
        assert!(
            claim_res.sector_results.all_ok() || claim_res.sector_results.success_count == 0,
            "batch return of claim allocations partially succeeded but request was all_or_nothing {:?}",
            claim_res
        );
    }

    let activation_outputs = claim_res
        .sector_claims
        .iter()
        .zip(claim_res.sector_results.successes(&sectors_pieces))
        .map(|(sector_claim, sector_pieces)| {
            let mut unverified_space = BigInt::zero();
            let mut pieces = Vec::new();
            for piece in *sector_pieces {
                if piece.verified_allocation_key.is_none() {
                    unverified_space += piece.size.0;
                }
                pieces.push((piece.cid, piece.size.0));
            }
            DataActivationOutput {
                unverified_space: unverified_space.clone(),
                verified_space: sector_claim.claimed_space.clone(),
                unsealed_cid: None,
                pieces,
            }
        })
        .collect();

    Ok((claim_res.sector_results, activation_outputs))
}

/// Activates deals then claims allocations for any verified deals
/// Deals and claims are grouped by sectors
/// Successfully activated sectors have their DealSpaces returned
/// Failure to claim datacap for any verified deal results in the whole batch failing
fn activate_sectors_deals(
    rt: &impl Runtime,
    activation_infos: &[DealsActivationInput],
    compute_unsealed_cid: bool,
) -> Result<(BatchReturn, Vec<DataActivationOutput>), ActorError> {
    let batch_activation_res = match activation_infos.iter().all(|p| p.deal_ids.is_empty()) {
        true => ext::market::BatchActivateDealsResult {
            // if all sectors are empty of deals, skip calling the market actor
            activations: vec![
                ext::market::SectorDealActivation {
                    activated: Vec::default(),
                    unsealed_cid: None,
                };
                activation_infos.len()
            ],
            activation_results: BatchReturn::ok(activation_infos.len() as u32),
        },
        false => {
            let sector_activation_params = activation_infos
                .iter()
                .map(|activation_info| ext::market::SectorDeals {
                    sector_number: activation_info.sector_number,
                    deal_ids: activation_info.deal_ids.clone(),
                    sector_expiry: activation_info.sector_expiry,
                    sector_type: activation_info.sector_type,
                })
                .collect();
            let activate_raw = extract_send_result(rt.send_simple(
                &STORAGE_MARKET_ACTOR_ADDR,
                ext::market::BATCH_ACTIVATE_DEALS_METHOD,
                IpldBlock::serialize_cbor(&ext::market::BatchActivateDealsParams {
                    sectors: sector_activation_params,
                    compute_cid: compute_unsealed_cid,
                })?,
                TokenAmount::zero(),
            ))?;
            deserialize_block::<ext::market::BatchActivateDealsResult>(activate_raw)?
        }
    };

    // When all prove commits have failed abort early
    if batch_activation_res.activation_results.success_count == 0 {
        return Err(actor_error_v14!(illegal_argument, "all deals failed to activate"));
    }

    // Filter the DealsActivationInfo for successfully activated sectors
    let successful_activation_infos =
        batch_activation_res.activation_results.successes(activation_infos);

    // Get a flattened list of verified claims for all activated sectors
    let mut verified_claims = Vec::new();
    for (activation_info, activate_res) in
        successful_activation_infos.iter().zip(&batch_activation_res.activations)
    {
        let sector_claims = activate_res
            .activated
            .iter()
            .filter(|info| info.allocation_id != NO_ALLOCATION_ID)
            .map(|info| ext::verifreg::AllocationClaim {
                client: info.client,
                allocation_id: info.allocation_id,
                data: info.data,
                size: info.size,
            })
            .collect();

        verified_claims.push(ext::verifreg::SectorAllocationClaims {
            sector: activation_info.sector_number,
            expiry: activation_info.sector_expiry,
            claims: sector_claims,
        });
    }

    let all_or_nothing = true;
    let claim_res = batch_claim_allocations(rt, verified_claims, all_or_nothing)?;
    assert!(
        claim_res.sector_results.all_ok() || claim_res.sector_results.success_count == 0,
        "batch return of claim allocations partially succeeded but request was all_or_nothing {:?}",
        claim_res
    );

    // reassociate the verified claims with corresponding DealActivation information
    let activation_and_claim_results = batch_activation_res
        .activations
        .iter()
        .zip(claim_res.sector_claims)
        .map(|(sector_deals, sector_claim)| {
            let mut sector_pieces = Vec::new();
            let mut unverified_deal_space = BigInt::zero();
            for info in &sector_deals.activated {
                sector_pieces.push((info.data, info.size.0));
                if info.allocation_id == NO_ALLOCATION_ID {
                    unverified_deal_space += info.size.0;
                }
            }
            DataActivationOutput {
                unverified_space: unverified_deal_space,
                verified_space: sector_claim.claimed_space,
                unsealed_cid: sector_deals.unsealed_cid,
                pieces: sector_pieces,
            }
        })
        .collect();

    // Return the deal spaces for activated sectors only
    Ok((batch_activation_res.activation_results, activation_and_claim_results))
}

fn batch_claim_allocations(
    rt: &impl Runtime,
    verified_claims: Vec<ext::verifreg::SectorAllocationClaims>,
    all_or_nothing: bool,
) -> Result<ext::verifreg::ClaimAllocationsReturn, ActorError> {
    let claim_res = match verified_claims.iter().all(|sector| sector.claims.is_empty()) {
        // Short-circuit the call if there are no claims,
        // but otherwise send a group for each sector (even if empty) to ease association of results.
        true => ext::verifreg::ClaimAllocationsReturn {
            sector_results: BatchReturn::ok(verified_claims.len() as u32),
            sector_claims: vec![
                ext::verifreg::SectorClaimSummary { claimed_space: BigInt::zero() };
                verified_claims.len()
            ],
        },
        false => {
            let claim_raw = extract_send_result(rt.send_simple(
                &VERIFIED_REGISTRY_ACTOR_ADDR,
                ext::verifreg::CLAIM_ALLOCATIONS_METHOD,
                IpldBlock::serialize_cbor(&ext::verifreg::ClaimAllocationsParams {
                    sectors: verified_claims,
                    all_or_nothing,
                })?,
                TokenAmount::zero(),
            ))
            .context("error claiming allocations on batch")?;

            let claim_res: ext::verifreg::ClaimAllocationsReturn = deserialize_block(claim_raw)?;
            claim_res
        }
    };
    Ok(claim_res)
}

fn unsealed_cid_from_pieces(
    rt: &impl Runtime,
    pieces: &[PieceActivationManifest],
    sector_type: RegisteredSealProof,
) -> Result<CompactCommD, ActorError> {
    let computed_commd = if !pieces.is_empty() {
        let pieces: Vec<PieceInfo> =
            pieces.iter().map(|piece| PieceInfo { cid: piece.cid, size: piece.size }).collect();
        let computed = rt.compute_unsealed_sector_cid(sector_type, &pieces).context_code(
            ExitCode::USR_ILLEGAL_ARGUMENT,
            "failed to compute unsealed sector CID",
        )?;
        CompactCommD::of(computed)
    } else {
        CompactCommD::empty()
    };
    Ok(computed_commd)
}

// Network inputs to calculation of sector pledge and associated parameters.
struct NetworkPledgeInputs {
    pub network_qap: FilterEstimate,
    pub network_baseline: StoragePower,
    pub circulating_supply: TokenAmount,
    pub epoch_reward: FilterEstimate,
}

// Note: probably better to push this one level down into state
fn balance_invariants_broken(e: Error) -> ActorError {
    ActorError::unchecked(
        ERR_BALANCE_INVARIANTS_BROKEN,
        format!("balance invariants broken: {}", e),
    )
}
