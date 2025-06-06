// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::borrow::Borrow;
use std::cmp;
use std::ops::Neg;

use anyhow::anyhow;
use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::{BytesDe, CborStore, strict_bytes};
use fvm_shared4::address::Address;
use fvm_shared4::clock::{ChainEpoch, EPOCH_UNDEFINED};
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;
use fvm_shared4::sector::{RegisteredPoStProof, SectorNumber, SectorSize};
use fvm_shared4::{ActorID, HAMT_BIT_WIDTH};
use itertools::Itertools;
use multihash_codetable::Code;
use num_traits::Zero;

use fil_actors_shared::actor_error_v16;
use fil_actors_shared::v16::runtime::Policy;
use fil_actors_shared::v16::runtime::policy_constants::MAX_SECTOR_NUMBER;
use fil_actors_shared::v16::{
    ActorContext, ActorDowncast, ActorError, Array, AsActorError, Config, DEFAULT_HAMT_CONFIG, Map2,
};

use super::beneficiary::*;
use super::deadlines::new_deadline_info;
use super::policy::*;
use super::types::*;
use super::{
    BitFieldQueue, Deadline, DeadlineInfo, DeadlineSectorMap, Deadlines, PowerPair, QuantSpec,
    Sectors, TerminationResult, VestingFunds, assign_deadlines, deadline_is_mutable,
    new_deadline_info_from_offset_and_epoch, quant_spec_for_deadline,
};

pub type PreCommitMap<BS> = Map2<BS, SectorNumber, SectorPreCommitOnChainInfo>;
pub const PRECOMMIT_CONFIG: Config = Config {
    bit_width: HAMT_BIT_WIDTH,
    ..DEFAULT_HAMT_CONFIG
};

const PRECOMMIT_EXPIRY_AMT_BITWIDTH: u32 = 6;
pub const SECTORS_AMT_BITWIDTH: u32 = 5;

/// Balance of Miner Actor should be greater than or equal to
/// the sum of PreCommitDeposits and LockedFunds.
/// It is possible for balance to fall below the sum of PCD, LF and
/// InitialPledgeRequirements, and this is a bad state (IP Debt)
/// that limits a miner actor's behavior (i.e. no balance withdrawals)
/// Excess balance as computed by st.GetAvailableBalance will be
/// withdrawable or usable for pre-commit deposit or pledge lock-up.
#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct State {
    /// Contains static info about this miner
    pub info: Cid,

    /// Total funds locked as pre_commit_deposit
    pub pre_commit_deposits: TokenAmount,

    /// Total rewards and added funds locked in vesting table
    pub locked_funds: TokenAmount,

    /// VestingFunds (Vesting Funds schedule for the miner).
    pub vesting_funds: VestingFunds,

    /// Absolute value of debt this miner owes from unpaid fees.
    pub fee_debt: TokenAmount,

    /// Sum of initial pledge requirements of all active sectors.
    pub initial_pledge: TokenAmount,

    /// Sectors that have been pre-committed but not yet proven.
    /// Map, HAMT<SectorNumber, SectorPreCommitOnChainInfo>
    pub pre_committed_sectors: Cid,

    // PreCommittedSectorsCleanUp maintains the state required to cleanup expired PreCommittedSectors.
    pub pre_committed_sectors_cleanup: Cid, // BitFieldQueue (AMT[Epoch]*BitField)

    /// Allocated sector IDs. Sector IDs can never be reused once allocated.
    pub allocated_sectors: Cid, // BitField

    /// Information for all proven and not-yet-garbage-collected sectors.
    ///
    /// Sectors are removed from this AMT when the partition to which the
    /// sector belongs is compacted.
    pub sectors: Cid, // Array, AMT[SectorNumber]SectorOnChainInfo (sparse)

    /// The first epoch in this miner's current proving period. This is the first epoch in which a PoSt for a
    /// partition at the miner's first deadline may arrive. Alternatively, it is after the last epoch at which
    /// a PoSt for the previous window is valid.
    /// Always greater than zero, this may be greater than the current epoch for genesis miners in the first
    /// WPoStProvingPeriod epochs of the chain; the epochs before the first proving period starts are exempt from Window
    /// PoSt requirements.
    /// Updated at the end of every period by a cron callback.
    pub proving_period_start: ChainEpoch,

    /// Index of the deadline within the proving period beginning at ProvingPeriodStart that has not yet been
    /// finalized.
    /// Updated at the end of each deadline window by a cron callback.
    pub current_deadline: u64,

    /// The sector numbers due for PoSt at each deadline in the current proving period, frozen at period start.
    /// New sectors are added and expired ones removed at proving period boundary.
    /// Faults are not subtracted from this in state, but on the fly.
    pub deadlines: Cid,

    /// Deadlines with outstanding fees for early sector termination.
    pub early_terminations: BitField,

    // True when miner cron is active, false otherwise
    pub deadline_cron_active: bool,
}

#[derive(PartialEq, Eq)]
pub enum CollisionPolicy {
    AllowCollisions,
    DenyCollisions,
}

impl State {
    #[allow(clippy::too_many_arguments)]
    pub fn new<BS: Blockstore>(
        policy: &Policy,
        store: &BS,
        info_cid: Cid,
        period_start: ChainEpoch,
        deadline_idx: u64,
    ) -> Result<Self, ActorError> {
        let empty_precommit_map =
            PreCommitMap::empty(store, PRECOMMIT_CONFIG, "precommits").flush()?;

        let empty_precommits_cleanup_array =
            Array::<BitField, BS>::new_with_bit_width(store, PRECOMMIT_EXPIRY_AMT_BITWIDTH)
                .flush()
                .map_err(|e| {
                    e.downcast_default(
                        ExitCode::USR_ILLEGAL_STATE,
                        "failed to construct empty precommits array",
                    )
                })?;
        let empty_sectors_array =
            Array::<SectorOnChainInfo, BS>::new_with_bit_width(store, SECTORS_AMT_BITWIDTH)
                .flush()
                .map_err(|e| {
                    e.downcast_default(
                        ExitCode::USR_ILLEGAL_STATE,
                        "failed to construct sectors array",
                    )
                })?;
        let empty_bitfield = store
            .put_cbor(&BitField::new(), Code::Blake2b256)
            .map_err(|e| {
                e.downcast_default(
                    ExitCode::USR_ILLEGAL_STATE,
                    "failed to construct empty bitfield",
                )
            })?;
        let deadline = Deadline::new(store)?;
        let empty_deadline = store.put_cbor(&deadline, Code::Blake2b256).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct illegal state",
            )
        })?;

        let empty_deadlines = store
            .put_cbor(&Deadlines::new(policy, empty_deadline), Code::Blake2b256)
            .map_err(|e| {
                e.downcast_default(
                    ExitCode::USR_ILLEGAL_STATE,
                    "failed to construct illegal state",
                )
            })?;

        Ok(Self {
            info: info_cid,

            pre_commit_deposits: TokenAmount::default(),
            locked_funds: TokenAmount::default(),

            vesting_funds: VestingFunds::new(),

            initial_pledge: TokenAmount::default(),
            fee_debt: TokenAmount::default(),

            pre_committed_sectors: empty_precommit_map,
            allocated_sectors: empty_bitfield,
            sectors: empty_sectors_array,
            proving_period_start: period_start,
            current_deadline: deadline_idx,
            deadlines: empty_deadlines,
            early_terminations: BitField::new(),
            deadline_cron_active: false,
            pre_committed_sectors_cleanup: empty_precommits_cleanup_array,
        })
    }

    pub fn get_info<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<MinerInfo> {
        match store.get_cbor(&self.info) {
            Ok(Some(info)) => Ok(info),
            Ok(None) => Err(actor_error_v16!(not_found, "failed to get miner info").into()),
            Err(e) => Err(e.downcast_wrap("failed to get miner info")),
        }
    }

    pub fn save_info<BS: Blockstore>(
        &mut self,
        store: &BS,
        info: &MinerInfo,
    ) -> anyhow::Result<()> {
        let cid = store.put_cbor(&info, Code::Blake2b256)?;
        self.info = cid;
        Ok(())
    }

    /// Returns deadline calculations for the current (according to state) proving period.
    pub fn deadline_info(&self, policy: &Policy, current_epoch: ChainEpoch) -> DeadlineInfo {
        new_deadline_info_from_offset_and_epoch(policy, self.proving_period_start, current_epoch)
    }
    // Returns deadline calculations for the state recorded proving period and deadline.
    // This is out of date if the a miner does not have an active miner cron
    pub fn recorded_deadline_info(
        &self,
        policy: &Policy,
        current_epoch: ChainEpoch,
    ) -> DeadlineInfo {
        new_deadline_info(
            policy,
            self.proving_period_start,
            self.current_deadline,
            current_epoch,
        )
    }

    // Returns current proving period start for the current epoch according to the current epoch and constant state offset
    pub fn current_proving_period_start(
        &self,
        policy: &Policy,
        current_epoch: ChainEpoch,
    ) -> ChainEpoch {
        let dl_info = self.deadline_info(policy, current_epoch);
        dl_info.period_start
    }

    /// Returns deadline calculations for the current (according to state) proving period.
    pub fn quant_spec_for_deadline(&self, policy: &Policy, deadline_idx: u64) -> QuantSpec {
        new_deadline_info(policy, self.proving_period_start, deadline_idx, 0).quant_spec()
    }

    /// Marks a set of sector numbers as having been allocated.
    /// If policy is `DenyCollisions`, fails if the set intersects with the sector numbers already allocated.
    pub fn allocate_sector_numbers<BS: Blockstore>(
        &mut self,
        store: &BS,
        sector_numbers: &BitField,
        policy: CollisionPolicy,
    ) -> Result<(), ActorError> {
        let prior_allocation = store
            .get_cbor(&self.allocated_sectors)
            .map_err(|e| {
                e.downcast_default(
                    ExitCode::USR_ILLEGAL_STATE,
                    "failed to load allocated sectors bitfield",
                )
            })?
            .ok_or_else(|| {
                actor_error_v16!(illegal_state, "allocated sectors bitfield not found")
            })?;

        if policy != CollisionPolicy::AllowCollisions {
            // NOTE: A fancy merge algorithm could extract this intersection while merging, below, saving
            // one iteration of the runs
            let collisions = &prior_allocation & sector_numbers;
            if !collisions.is_empty() {
                return Err(actor_error_v16!(
                    illegal_argument,
                    "sector numbers {:?} already allocated",
                    collisions
                ));
            }
        }
        let new_allocation = &prior_allocation | sector_numbers;
        self.allocated_sectors =
            store
                .put_cbor(&new_allocation, Code::Blake2b256)
                .map_err(|e| {
                    e.downcast_default(
                        ExitCode::USR_ILLEGAL_ARGUMENT,
                        format!(
                            "failed to store allocated sectors bitfield after adding {:?}",
                            sector_numbers,
                        ),
                    )
                })?;
        Ok(())
    }

    /// Stores a pre-committed sector info, failing if the sector number is already present.
    pub fn put_precommitted_sectors<BS: Blockstore>(
        &mut self,
        store: &BS,
        precommits: Vec<SectorPreCommitOnChainInfo>,
    ) -> anyhow::Result<()> {
        let mut precommitted = PreCommitMap::load(
            store,
            &self.pre_committed_sectors,
            PRECOMMIT_CONFIG,
            "precommits",
        )?;
        for precommit in precommits.into_iter() {
            let sector_no = precommit.info.sector_number;
            let modified = precommitted
                .set_if_absent(&sector_no, precommit)
                .with_context(|| format!("storing precommit for {}", sector_no))?;
            if !modified {
                return Err(anyhow!("sector {} already pre-commited", sector_no));
            }
        }

        self.pre_committed_sectors = precommitted.flush()?;
        Ok(())
    }

    pub fn get_precommitted_sector<BS: Blockstore>(
        &self,
        store: &BS,
        sector_num: SectorNumber,
    ) -> Result<Option<SectorPreCommitOnChainInfo>, ActorError> {
        let precommitted = PreCommitMap::load(
            store,
            &self.pre_committed_sectors,
            PRECOMMIT_CONFIG,
            "precommits",
        )?;
        Ok(precommitted.get(&sector_num)?.cloned())
    }

    /// Gets and returns the requested pre-committed sectors, skipping missing sectors.
    pub fn find_precommitted_sectors<BS: Blockstore>(
        &self,
        store: &BS,
        sector_numbers: &[SectorNumber],
    ) -> anyhow::Result<Vec<SectorPreCommitOnChainInfo>> {
        let precommitted = PreCommitMap::load(
            store,
            &self.pre_committed_sectors,
            PRECOMMIT_CONFIG,
            "precommits",
        )?;
        let mut result = Vec::with_capacity(sector_numbers.len());

        for &sector_number in sector_numbers {
            let info = match precommitted
                .get(&sector_number)
                .with_context(|| format!("loading precommit {}", sector_number))?
            {
                Some(info) => info.clone(),
                None => continue,
            };

            result.push(info);
        }

        Ok(result)
    }

    pub fn delete_precommitted_sectors<BS: Blockstore>(
        &mut self,
        store: &BS,
        sector_nums: &[SectorNumber],
    ) -> Result<(), ActorError> {
        let mut precommitted = PreCommitMap::load(
            store,
            &self.pre_committed_sectors,
            PRECOMMIT_CONFIG,
            "precommits",
        )?;
        for &sector_num in sector_nums {
            let prev_entry = precommitted.delete(&sector_num)?;
            if prev_entry.is_none() {
                return Err(actor_error_v16!(
                    illegal_state,
                    "sector {} not pre-committed",
                    sector_num
                ));
            }
        }

        self.pre_committed_sectors = precommitted.flush()?;
        Ok(())
    }

    pub fn has_sector_number<BS: Blockstore>(
        &self,
        store: &BS,
        sector_num: SectorNumber,
    ) -> anyhow::Result<bool> {
        let sectors = Sectors::load(store, &self.sectors)?;
        Ok(sectors.get(sector_num)?.is_some())
    }

    pub fn put_sectors<BS: Blockstore>(
        &mut self,
        store: &BS,
        new_sectors: Vec<SectorOnChainInfo>,
    ) -> anyhow::Result<()> {
        let mut sectors = Sectors::load(store, &self.sectors)
            .map_err(|e| e.downcast_wrap("failed to load sectors"))?;

        sectors.store(new_sectors)?;

        self.sectors = sectors
            .amt
            .flush()
            .map_err(|e| e.downcast_wrap("failed to persist sectors"))?;

        Ok(())
    }

    pub fn get_sector<BS: Blockstore>(
        &self,
        store: &BS,
        sector_num: SectorNumber,
    ) -> Result<Option<SectorOnChainInfo>, ActorError> {
        let sectors = Sectors::load(store, &self.sectors)
            .context_code(ExitCode::USR_ILLEGAL_STATE, "loading sectors")?;
        sectors.get(sector_num)
    }

    pub fn for_each_sector<BS: Blockstore, F>(&self, store: &BS, mut f: F) -> anyhow::Result<()>
    where
        F: FnMut(&SectorOnChainInfo) -> anyhow::Result<()>,
    {
        let sectors = Sectors::load(store, &self.sectors)?;
        sectors.amt.for_each(|_, v| f(v))?;
        Ok(())
    }

    /// Returns the deadline and partition index for a sector number.
    pub fn find_sector<BS: Blockstore>(
        &self,
        store: &BS,
        sector_number: SectorNumber,
    ) -> anyhow::Result<(u64, u64)> {
        let deadlines = self.load_deadlines(store)?;
        deadlines.find_sector(store, sector_number)
    }

    /// Schedules each sector to expire at its next deadline end. If it can't find
    /// any given sector, it skips it.
    ///
    /// This method assumes that each sector's power has not changed, despite the rescheduling.
    ///
    /// Note: this method is used to "upgrade" sectors, rescheduling the now-replaced
    /// sectors to expire at the end of the next deadline. Given the expense of
    /// sealing a sector, this function skips missing/faulty/terminated "upgraded"
    /// sectors instead of failing. That way, the new sectors can still be proved.
    pub fn reschedule_sector_expirations<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        current_epoch: ChainEpoch,
        sector_size: SectorSize,
        mut deadline_sectors: DeadlineSectorMap,
    ) -> anyhow::Result<Vec<SectorOnChainInfo>> {
        let mut deadlines = self.load_deadlines(store)?;
        let sectors = Sectors::load(store, &self.sectors)?;

        let mut all_replaced = Vec::new();
        for (deadline_idx, partition_sectors) in deadline_sectors.iter() {
            let deadline_info = new_deadline_info(
                policy,
                self.current_proving_period_start(policy, current_epoch),
                deadline_idx,
                current_epoch,
            )
            .next_not_elapsed();
            let new_expiration = deadline_info.last();
            let mut deadline = deadlines.load_deadline(store, deadline_idx)?;

            let replaced = deadline.reschedule_sector_expirations(
                store,
                &sectors,
                new_expiration,
                partition_sectors,
                sector_size,
                deadline_info.quant_spec(),
            )?;
            all_replaced.extend(replaced);

            deadlines.update_deadline(policy, store, deadline_idx, &deadline)?;
        }

        self.save_deadlines(store, deadlines)?;

        Ok(all_replaced)
    }

    /// Assign new sectors to deadlines.
    pub fn assign_sectors_to_deadlines<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        current_epoch: ChainEpoch,
        mut sectors: Vec<SectorOnChainInfo>,
        partition_size: u64,
        sector_size: SectorSize,
    ) -> anyhow::Result<()> {
        let mut deadlines = self.load_deadlines(store)?;

        // Sort sectors by number to get better runs in partition bitfields.
        sectors.sort_by_key(|info| info.sector_number);

        let mut deadline_vec: Vec<Option<Deadline>> =
            (0..policy.wpost_period_deadlines).map(|_| None).collect();

        deadlines.for_each(store, |deadline_idx, deadline| {
            // Skip deadlines that aren't currently mutable.
            if deadline_is_mutable(
                policy,
                self.current_proving_period_start(policy, current_epoch),
                deadline_idx,
                current_epoch,
            ) {
                deadline_vec[deadline_idx as usize] = Some(deadline);
            }

            Ok(())
        })?;

        let deadline_to_sectors = assign_deadlines(
            policy,
            policy.max_partitions_per_deadline,
            partition_size,
            &deadline_vec,
            sectors,
        )?;

        for (deadline_idx, deadline_sectors) in deadline_to_sectors.into_iter().enumerate() {
            if deadline_sectors.is_empty() {
                continue;
            }

            let quant = self.quant_spec_for_deadline(policy, deadline_idx as u64);
            let deadline = deadline_vec[deadline_idx].as_mut().unwrap();

            // The power returned from AddSectors is ignored because it's not activated (proven) yet.
            let proven = false;
            // New sectors, so the deadline has new fees.
            let new_fees = true;
            deadline.add_sectors(
                store,
                partition_size,
                proven,
                new_fees,
                &deadline_sectors,
                sector_size,
                quant,
            )?;

            deadlines.update_deadline(policy, store, deadline_idx as u64, deadline)?;
        }

        self.save_deadlines(store, deadlines)?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assign_sectors_to_deadline<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        current_epoch: ChainEpoch,
        mut sectors: Vec<SectorOnChainInfo>,
        partition_size: u64,
        sector_size: SectorSize,
        deadline_idx: u64,
    ) -> Result<(), ActorError> {
        let mut deadlines = self.load_deadlines(store)?;
        let mut deadline = deadlines.load_deadline(store, deadline_idx)?;

        // Sort sectors by number to get better runs in partition bitfields.
        sectors.sort_by_key(|info| info.sector_number);

        if !deadline_is_mutable(
            policy,
            self.current_proving_period_start(policy, current_epoch),
            deadline_idx,
            current_epoch,
        ) {
            return Err(actor_error_v16!(
                illegal_argument,
                "proving deadline {} must not be the current or next deadline ",
                deadline_idx
            ));
        }

        let quant = self.quant_spec_for_deadline(policy, deadline_idx);
        let proven = false;
        let new_fees = true; // New sectors, so the deadline has new fees.
        deadline
            .add_sectors(
                store,
                partition_size,
                proven,
                new_fees,
                &sectors,
                sector_size,
                quant,
            )
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to add sectors to deadline {}", deadline_idx)
            })?;

        deadlines
            .update_deadline(policy, store, deadline_idx, &deadline)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to update deadline {}", deadline_idx)
            })?;
        self.save_deadlines(store, deadlines)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || "failed to save deadlines")?;

        Ok(())
    }

    /// Pops up to `max_sectors` early terminated sectors from all deadlines.
    ///
    /// Returns `true` if we still have more early terminations to process.
    pub fn pop_early_terminations<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        max_partitions: u64,
        max_sectors: u64,
    ) -> anyhow::Result<(TerminationResult, /* has more */ bool)> {
        // Anything to do? This lets us avoid loading the deadlines if there's nothing to do.
        if self.early_terminations.is_empty() {
            return Ok((Default::default(), false));
        }

        // Load deadlines
        let mut deadlines = self.load_deadlines(store)?;

        let mut result = TerminationResult::new();
        let mut to_unset = Vec::new();

        // Process early terminations.
        for i in self.early_terminations.iter() {
            let deadline_idx = i;

            // Load deadline + partitions.
            let mut deadline = deadlines.load_deadline(store, deadline_idx)?;

            let (deadline_result, more) = deadline
                .pop_early_terminations(
                    store,
                    max_partitions - result.partitions_processed,
                    max_sectors - result.sectors_processed,
                )
                .map_err(|e| {
                    e.downcast_wrap(format!(
                        "failed to pop early terminations for deadline {}",
                        deadline_idx
                    ))
                })?;

            result += deadline_result;

            if !more {
                to_unset.push(i);
            }

            // Save the deadline
            deadlines.update_deadline(policy, store, deadline_idx, &deadline)?;

            if !result.below_limit(max_partitions, max_sectors) {
                break;
            }
        }

        for deadline_idx in to_unset {
            self.early_terminations.unset(deadline_idx);
        }

        // Save back the deadlines.
        self.save_deadlines(store, deadlines)?;

        // Ok, check to see if we've handled all early terminations.
        let no_early_terminations = self.early_terminations.is_empty();

        Ok((result, !no_early_terminations))
    }

    /// Returns an error if the target sector cannot be found, or some other bad state is reached.
    /// Returns Ok(false) if the target sector is faulty, terminated, or unproven
    /// Returns Ok(true) otherwise
    pub fn check_sector_active<BS: Blockstore>(
        &self,
        store: &BS,
        deadline_idx: u64,
        partition_idx: u64,
        sector_number: SectorNumber,
        require_proven: bool,
    ) -> Result<bool, ActorError> {
        let dls = self.load_deadlines(store)?;
        let dl = dls.load_deadline(store, deadline_idx)?;
        let partition = dl.load_partition(store, partition_idx)?;

        let exists = partition.sectors.get(sector_number);
        if !exists {
            return Err(actor_error_v16!(
                not_found;
                "sector {} not a member of partition {}, deadline {}",
                sector_number, partition_idx, deadline_idx
            ));
        }

        let faulty = partition.faults.get(sector_number);
        if faulty {
            return Ok(false);
        }

        let terminated = partition.terminated.get(sector_number);
        if terminated {
            return Ok(false);
        }

        let unproven = partition.unproven.get(sector_number);
        if unproven && require_proven {
            return Ok(false);
        }

        Ok(true)
    }

    /// Returns an error if the target sector cannot be found and/or is faulty/terminated.
    pub fn check_sector_health<BS: Blockstore>(
        &self,
        store: &BS,
        deadline_idx: u64,
        partition_idx: u64,
        sector_number: SectorNumber,
    ) -> anyhow::Result<()> {
        let deadlines = self.load_deadlines(store)?;
        let deadline = deadlines.load_deadline(store, deadline_idx)?;
        let partition = deadline.load_partition(store, partition_idx)?;

        if !partition.sectors.get(sector_number) {
            return Err(actor_error_v16!(
                not_found;
                "sector {} not a member of partition {}, deadline {}",
                sector_number, partition_idx, deadline_idx
            )
            .into());
        }

        if partition.faults.get(sector_number) {
            return Err(actor_error_v16!(
                forbidden;
                "sector {} not a member of partition {}, deadline {}",
                sector_number, partition_idx, deadline_idx
            )
            .into());
        }

        if partition.terminated.get(sector_number) {
            return Err(actor_error_v16!(
                not_found;
                "sector {} not of partition {}, deadline {} is terminated",
                sector_number, partition_idx, deadline_idx
            )
            .into());
        }

        Ok(())
    }

    /// Loads sector info for a sequence of sectors.
    pub fn load_sector_infos<BS: Blockstore>(
        &self,
        store: &BS,
        sectors: &BitField,
    ) -> anyhow::Result<Vec<SectorOnChainInfo>> {
        Ok(Sectors::load(store, &self.sectors)?.load_sectors(sectors)?)
    }

    pub fn load_deadlines<BS: Blockstore>(&self, store: &BS) -> Result<Deadlines, ActorError> {
        store
            .get_cbor::<Deadlines>(&self.deadlines)
            .map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load deadlines")
            })?
            .ok_or_else(
                || actor_error_v16!(illegal_state; "failed to load deadlines {}", self.deadlines),
            )
    }

    pub fn save_deadlines<BS: Blockstore>(
        &mut self,
        store: &BS,
        deadlines: Deadlines,
    ) -> anyhow::Result<()> {
        self.deadlines = store.put_cbor(&deadlines, Code::Blake2b256)?;
        Ok(())
    }

    // Return true when the miner actor needs to continue scheduling deadline crons
    pub fn continue_deadline_cron(&self) -> bool {
        !self.pre_commit_deposits.is_zero()
            || !self.initial_pledge.is_zero()
            || !self.locked_funds.is_zero()
    }

    //
    // Funds and vesting
    //

    pub fn add_pre_commit_deposit(&mut self, amount: &TokenAmount) -> anyhow::Result<()> {
        let new_total = &self.pre_commit_deposits + amount;
        if new_total.is_negative() {
            return Err(anyhow!(
                "negative pre-commit deposit {} after adding {} to prior {}",
                new_total,
                amount,
                self.pre_commit_deposits
            ));
        }
        self.pre_commit_deposits = new_total;
        Ok(())
    }

    pub fn add_initial_pledge(&mut self, amount: &TokenAmount) -> anyhow::Result<()> {
        let new_total = &self.initial_pledge + amount;
        if new_total.is_negative() {
            return Err(anyhow!(
                "negative initial pledge requirement {} after adding {} to prior {}",
                new_total,
                amount,
                self.initial_pledge
            ));
        }
        self.initial_pledge = new_total;
        Ok(())
    }

    pub fn apply_penalty(&mut self, penalty: &TokenAmount) -> anyhow::Result<()> {
        if penalty.is_negative() {
            Err(anyhow!("applying negative penalty {} not allowed", penalty))
        } else {
            self.fee_debt += penalty;
            Ok(())
        }
    }

    /// First vests and unlocks the vested funds AND then locks the given funds in the vesting table.
    pub fn add_locked_funds<BS: Blockstore>(
        &mut self,
        store: &BS,
        current_epoch: ChainEpoch,
        vesting_sum: &TokenAmount,
        spec: &VestSpec,
    ) -> anyhow::Result<TokenAmount> {
        if vesting_sum.is_negative() {
            return Err(anyhow!("negative vesting sum {}", vesting_sum));
        }
        // add new funds and unlock already vested funds.
        let amount_unlocked = self.vesting_funds.add_locked_funds(
            store,
            current_epoch,
            vesting_sum,
            self.proving_period_start,
            spec,
        )?;

        // We shouldn't unlock any of the new funds, so the locked funds should remain non-negative
        // when we deduct the amount unlocked.
        self.locked_funds -= &amount_unlocked;
        if self.locked_funds.is_negative() {
            return Err(anyhow!(
                "negative locked funds {} after unlocking {}",
                self.locked_funds,
                amount_unlocked
            ));
        }

        // Finally, record the new locked-funds total.
        self.locked_funds += vesting_sum;

        Ok(amount_unlocked)
    }

    /// Draws from vesting table and unlocked funds to repay up to the fee debt.
    /// Returns the amount to burn and the total amount unlocked from the vesting table (both vested
    /// and unvested) If the fee debt exceeds the total amount available for repayment the fee debt
    /// field is updated to track the remaining debt. Otherwise it is set to zero.
    pub fn repay_partial_debt_in_priority_order<BS: Blockstore>(
        &mut self,
        store: &BS,
        current_epoch: ChainEpoch,
        curr_balance: &TokenAmount,
    ) -> Result<
        (
            TokenAmount, // fee to burn
            TokenAmount, // total unlocked
        ),
        anyhow::Error,
    > {
        let fee_debt = self.fee_debt.clone();
        let (from_vesting, total_unlocked) =
            self.unlock_vested_and_unvested_funds(store, current_epoch, &fee_debt)?;

        if from_vesting > self.fee_debt {
            return Err(anyhow!(
                "should never unlock more than the debt we need to repay"
            ));
        }

        // locked unvested funds should now have been moved to unlocked balance if
        // there was enough to cover the fee debt
        let unlocked_balance = self.get_unlocked_balance(curr_balance)?;
        let to_burn = cmp::min(&unlocked_balance, &self.fee_debt).clone();
        self.fee_debt -= &to_burn;

        Ok((to_burn, total_unlocked))
    }

    /// Repays the full miner actor fee debt.  Returns the amount that must be
    /// burnt and an error if there are not sufficient funds to cover repayment.
    /// Miner state repays from unlocked funds and fails if unlocked funds are insufficient to cover fee debt.
    /// FeeDebt will be zero after a successful call.
    pub fn repay_debts(&mut self, curr_balance: &TokenAmount) -> anyhow::Result<TokenAmount> {
        let unlocked_balance = self.get_unlocked_balance(curr_balance)?;
        if unlocked_balance < self.fee_debt {
            return Err(actor_error_v16!(
                insufficient_funds,
                "unlocked balance can not repay fee debt ({} < {})",
                unlocked_balance,
                self.fee_debt
            )
            .into());
        }

        Ok(std::mem::take(&mut self.fee_debt))
    }

    /// Unlocks all vested funds and then unlocks an amount of funds that have *not yet vested*, if
    /// possible. The soonest-vesting entries are unlocked first.
    ///
    /// Returns the amount of unvested funds unlocked, along with the total amount of funds unlocked.
    pub fn unlock_vested_and_unvested_funds<BS: Blockstore>(
        &mut self,
        store: &BS,
        current_epoch: ChainEpoch,
        target: &TokenAmount,
    ) -> anyhow::Result<(
        TokenAmount, // unlocked_unvested
        TokenAmount, // total_unlocked
    )> {
        if target.is_zero() || self.locked_funds.is_zero() {
            return Ok((TokenAmount::zero(), TokenAmount::zero()));
        }

        let (unlocked_vested, unlocked_unvested) = self
            .vesting_funds
            .unlock_vested_and_unvested_funds(store, current_epoch, target)?;
        let total_unlocked = &unlocked_vested + &unlocked_unvested;
        self.locked_funds -= &total_unlocked;
        if self.locked_funds.is_negative() {
            return Err(anyhow!(
                "negative locked funds {} after unlocking {}",
                self.locked_funds,
                total_unlocked,
            ));
        }

        Ok((unlocked_unvested, total_unlocked))
    }

    /// Unlocks all vesting funds that have vested before the provided epoch.
    /// Returns the amount unlocked.
    pub fn unlock_vested_funds<BS: Blockstore>(
        &mut self,
        store: &BS,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<TokenAmount> {
        if self.locked_funds.is_zero() {
            return Ok(TokenAmount::zero());
        }

        let amount_unlocked = self
            .vesting_funds
            .unlock_vested_funds(store, current_epoch)?;
        self.locked_funds -= &amount_unlocked;
        if self.locked_funds.is_negative() {
            return Err(anyhow!(
                "vesting cause locked funds to become negative: {}",
                self.locked_funds,
            ));
        }

        Ok(amount_unlocked)
    }

    /// CheckVestedFunds returns the amount of vested funds that have vested before the provided epoch.
    pub fn check_vested_funds<BS: Blockstore>(
        &self,
        store: &BS,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<TokenAmount> {
        let vesting_funds = self.vesting_funds.load(store)?;
        Ok(vesting_funds
            .iter()
            .take_while(|fund| fund.epoch < current_epoch)
            .fold(TokenAmount::zero(), |acc, fund| acc + &fund.amount))
    }

    /// Unclaimed funds that are not locked -- includes funds used to cover initial pledge requirement.
    pub fn get_unlocked_balance(&self, actor_balance: &TokenAmount) -> anyhow::Result<TokenAmount> {
        let unlocked_balance =
            actor_balance - &self.locked_funds - &self.pre_commit_deposits - &self.initial_pledge;
        if unlocked_balance.is_negative() {
            return Err(anyhow!("negative unlocked balance {}", unlocked_balance));
        }
        Ok(unlocked_balance)
    }

    /// Unclaimed funds. Actor balance - (locked funds, precommit deposit, ip requirement)
    /// Can go negative if the miner is in IP debt.
    pub fn get_available_balance(
        &self,
        actor_balance: &TokenAmount,
    ) -> anyhow::Result<TokenAmount> {
        // (actor_balance - &self.locked_funds) - &self.pre_commit_deposit - &self.initial_pledge
        Ok(self.get_unlocked_balance(actor_balance)? - &self.fee_debt)
    }

    pub fn check_balance_invariants(&self, balance: &TokenAmount) -> anyhow::Result<()> {
        if self.pre_commit_deposits.is_negative() {
            return Err(anyhow!(
                "pre-commit deposit is negative: {}",
                self.pre_commit_deposits
            ));
        }
        if self.locked_funds.is_negative() {
            return Err(anyhow!("locked funds is negative: {}", self.locked_funds));
        }
        if self.initial_pledge.is_negative() {
            return Err(anyhow!(
                "initial pledge is negative: {}",
                self.initial_pledge
            ));
        }
        if self.fee_debt.is_negative() {
            return Err(anyhow!("fee debt is negative: {}", self.fee_debt));
        }

        let min_balance = &self.pre_commit_deposits + &self.locked_funds + &self.initial_pledge;
        if balance < &min_balance {
            return Err(anyhow!("balance {} below minimum {}", balance, min_balance));
        }

        Ok(())
    }

    /// pre-commit expiry
    pub fn quant_spec_every_deadline(&self, policy: &Policy) -> QuantSpec {
        QuantSpec {
            unit: policy.wpost_challenge_window,
            offset: self.proving_period_start,
        }
    }

    pub fn add_pre_commit_clean_ups<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        cleanup_events: Vec<(ChainEpoch, u64)>,
    ) -> anyhow::Result<()> {
        // Load BitField Queue for sector expiry
        let quant = self.quant_spec_every_deadline(policy);
        let mut queue =
            super::BitFieldQueue::new(store, &self.pre_committed_sectors_cleanup, quant)
                .map_err(|e| e.downcast_wrap("failed to load pre-commit clean up queue"))?;

        queue.add_many_to_queue_values(cleanup_events.into_iter())?;
        self.pre_committed_sectors_cleanup = queue.amt.flush()?;
        Ok(())
    }

    pub fn cleanup_expired_pre_commits<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<TokenAmount> {
        let mut deposit_to_burn = TokenAmount::zero();

        // cleanup expired pre-committed sectors
        let mut cleanup_queue = BitFieldQueue::new(
            store,
            &self.pre_committed_sectors_cleanup,
            self.quant_spec_every_deadline(policy),
        )?;

        let (sectors, modified) = cleanup_queue.pop_until(current_epoch)?;

        if modified {
            self.pre_committed_sectors_cleanup = cleanup_queue.amt.flush()?;
        }

        let mut precommits_to_delete = Vec::new();
        let precommitted = PreCommitMap::load(
            store,
            &self.pre_committed_sectors,
            PRECOMMIT_CONFIG,
            "precommits",
        )?;

        for i in sectors.iter() {
            let sector_number = i as SectorNumber;
            let sector: SectorPreCommitOnChainInfo =
                match precommitted.get(&sector_number)?.cloned() {
                    Some(sector) => sector,
                    // already committed/deleted
                    None => continue,
                };

            // mark it for deletion
            precommits_to_delete.push(sector_number);

            // increment deposit to burn
            deposit_to_burn += sector.pre_commit_deposit;
        }

        // Actually delete it.
        if !precommits_to_delete.is_empty() {
            self.delete_precommitted_sectors(store, &precommits_to_delete)?;
        }

        self.pre_commit_deposits -= &deposit_to_burn;
        if self.pre_commit_deposits.is_negative() {
            return Err(anyhow!(
                "pre-commit clean up caused negative deposits: {}",
                self.pre_commit_deposits
            ));
        }

        Ok(deposit_to_burn)
    }

    pub fn advance_deadline<BS: Blockstore>(
        &mut self,
        policy: &Policy,
        store: &BS,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<AdvanceDeadlineResult> {
        let mut pledge_delta = TokenAmount::zero();

        let dl_info = self.deadline_info(policy, current_epoch);

        if !dl_info.period_started() {
            return Ok(AdvanceDeadlineResult {
                pledge_delta,
                power_delta: PowerPair::zero(),
                previously_faulty_power: PowerPair::zero(),
                detected_faulty_power: PowerPair::zero(),
                total_faulty_power: PowerPair::zero(),
                daily_fee: TokenAmount::zero(),
                live_power: PowerPair::zero(),
            });
        }

        self.current_deadline = (dl_info.index + 1) % policy.wpost_period_deadlines;
        if self.current_deadline == 0 {
            self.proving_period_start = dl_info.period_start + policy.wpost_proving_period;
        }

        let mut deadlines = self.load_deadlines(store)?;

        let mut deadline = deadlines.load_deadline(store, dl_info.index)?;

        let previously_faulty_power = deadline.faulty_power.clone();

        if !deadline.is_live() {
            return Ok(AdvanceDeadlineResult {
                pledge_delta,
                power_delta: PowerPair::zero(),
                previously_faulty_power,
                detected_faulty_power: PowerPair::zero(),
                total_faulty_power: deadline.faulty_power,
                daily_fee: TokenAmount::zero(),
                live_power: PowerPair::zero(),
            });
        }

        let quant = quant_spec_for_deadline(policy, &dl_info);

        // Detect and penalize missing proofs.
        let fault_expiration = dl_info.last() + policy.fault_max_age;

        let (mut power_delta, detected_faulty_power) =
            deadline.process_deadline_end(store, quant, fault_expiration, self.sectors)?;

        // Capture deadline's faulty power after new faults have been detected, but before it is
        // dropped along with faulty sectors expiring this round.
        let total_faulty_power = deadline.faulty_power.clone();

        // Expire sectors that are due, either for on-time expiration or "early" faulty-for-too-long.
        let expired = deadline.pop_expired_sectors(store, dl_info.last(), quant)?;

        // Release pledge requirements for the sectors expiring on-time.
        // Pledge for the sectors expiring early is retained to support the termination fee that
        // will be assessed when the early termination is processed.
        pledge_delta -= &expired.on_time_pledge;
        self.add_initial_pledge(&expired.on_time_pledge.neg())?;

        // Record reduction in power of the amount of expiring active power.
        // Faulty power has already been lost, so the amount expiring can be excluded from the delta.
        power_delta -= &expired.active_power;

        let no_early_terminations = expired.early_sectors.is_empty();
        if !no_early_terminations {
            self.early_terminations.set(dl_info.index);
        }

        deadlines.update_deadline(policy, store, dl_info.index, &deadline)?;

        self.save_deadlines(store, deadlines)?;

        Ok(AdvanceDeadlineResult {
            pledge_delta,
            power_delta,
            previously_faulty_power,
            detected_faulty_power,
            total_faulty_power,
            daily_fee: deadline.daily_fee,
            live_power: deadline.live_power,
        })
    }

    // Loads sectors precommit information from store, requiring it to exist.
    pub fn get_precommitted_sectors<BS: Blockstore>(
        &self,
        store: &BS,
        sector_nos: impl IntoIterator<Item = impl Borrow<SectorNumber>>,
    ) -> Result<Vec<SectorPreCommitOnChainInfo>, ActorError> {
        let mut precommits = Vec::new();
        let precommitted = PreCommitMap::load(
            store,
            &self.pre_committed_sectors,
            PRECOMMIT_CONFIG,
            "precommits",
        )?;
        for sector_no in sector_nos.into_iter() {
            let sector_no = *sector_no.borrow();
            if sector_no > MAX_SECTOR_NUMBER {
                return Err(
                    actor_error_v16!(illegal_argument; "sector number greater than maximum"),
                );
            }
            let info: &SectorPreCommitOnChainInfo = precommitted
                .get(&sector_no)
                .exit_code(ExitCode::USR_ILLEGAL_STATE)?
                .ok_or_else(|| actor_error_v16!(not_found, "sector {} not found", sector_no))?;
            precommits.push(info.clone());
        }
        Ok(precommits)
    }
}

pub struct AdvanceDeadlineResult {
    pub pledge_delta: TokenAmount,
    pub power_delta: PowerPair,
    /// Power that was faulty before this advance (including recovering)
    pub previously_faulty_power: PowerPair,
    /// Power of new faults and failed recoveries
    pub detected_faulty_power: PowerPair,
    /// Total faulty power after detecting faults (before expiring sectors)
    /// Note that failed recovery power is included in both PreviouslyFaultyPower and
    /// DetectedFaultyPower, so TotalFaultyPower is not simply their sum.
    pub total_faulty_power: PowerPair,
    /// Fee payable for the sectors in the deadline being advanced
    pub daily_fee: TokenAmount,
    /// Total power for the deadline, including active, faulty, and unproven
    pub live_power: PowerPair,
}

/// Static information about miner
#[derive(Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct MinerInfo {
    /// Account that owns this miner
    /// - Income and returned collateral are paid to this address
    /// - This address is also allowed to change the worker address for the miner
    pub owner: Address,

    /// Worker account for this miner
    /// This will be the key that is used to sign blocks created by this miner, and
    /// sign messages sent on behalf of this miner to commit sectors, submit PoSts, and
    /// other day to day miner activities
    pub worker: Address,

    /// Additional addresses that are permitted to submit messages controlling this actor (optional).
    pub control_addresses: Vec<Address>, // Must all be ID addresses.

    /// Optional worker key to update at an epoch
    pub pending_worker_key: Option<WorkerKeyChange>,

    /// Libp2p identity that should be used when connecting to this miner
    #[serde(with = "strict_bytes")]
    pub peer_id: Vec<u8>,

    /// Vector of byte arrays representing Libp2p multi-addresses used for establishing a connection with this miner.
    pub multi_address: Vec<BytesDe>,

    /// The proof type used by this miner for sealing sectors.
    pub window_post_proof_type: RegisteredPoStProof,

    /// Amount of space in each sector committed to the network by this miner
    pub sector_size: SectorSize,

    /// The number of sectors in each Window PoSt partition (proof).
    /// This is computed from the proof type and represented here redundantly.
    pub window_post_partition_sectors: u64,

    /// The next epoch this miner is eligible for certain permissioned actor methods
    /// and winning block elections as a result of being reported for a consensus fault.
    pub consensus_fault_elapsed: ChainEpoch,

    /// A proposed new owner account for this miner.
    /// Must be confirmed by a message from the pending address itself.
    pub pending_owner_address: Option<Address>,

    /// Account for receive miner benefits, withdraw on miner must send to this address,
    /// set owner address by default when create miner
    pub beneficiary: Address,

    /// beneficiary's total quota, how much quota has been withdraw,
    /// and when this beneficiary expired
    pub beneficiary_term: BeneficiaryTerm,

    /// A proposal new beneficiary message for this miner
    pub pending_beneficiary_term: Option<PendingBeneficiaryChange>,
}

impl MinerInfo {
    pub fn new(
        owner: ActorID,
        worker: ActorID,
        control_addresses: Vec<ActorID>,
        peer_id: Vec<u8>,
        multi_address: Vec<BytesDe>,
        window_post_proof_type: RegisteredPoStProof,
    ) -> Result<Self, ActorError> {
        let sector_size = window_post_proof_type
            .sector_size()
            .map_err(|e| actor_error_v16!(illegal_argument, "invalid sector size: {}", e))?;

        let window_post_partition_sectors = window_post_proof_type
            .window_post_partition_sectors()
            .map_err(|e| actor_error_v16!(illegal_argument, "invalid partition sectors: {}", e))?;

        Ok(Self {
            owner: Address::new_id(owner),
            worker: Address::new_id(worker),
            control_addresses: control_addresses
                .into_iter()
                .map(Address::new_id)
                .collect_vec(),

            pending_worker_key: None,
            beneficiary: Address::new_id(owner),
            beneficiary_term: BeneficiaryTerm::default(),
            pending_beneficiary_term: None,
            peer_id,
            multi_address,
            window_post_proof_type,
            sector_size,
            window_post_partition_sectors,
            consensus_fault_elapsed: EPOCH_UNDEFINED,
            pending_owner_address: None,
        })
    }
}
