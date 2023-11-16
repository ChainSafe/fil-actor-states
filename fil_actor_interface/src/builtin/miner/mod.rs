// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::*;
use crate::Policy;
use anyhow::Context;
use cid::Cid;
use fvm_ipld_bitfield::BitField;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{serde_bytes, BytesDe};
use fvm_shared::{
    address::Address,
    clock::ChainEpoch,
    deal::DealID,
    econ::TokenAmount,
    sector::{RegisteredPoStProof, RegisteredSealProof, SectorNumber, SectorSize},
};
use lazy_static::lazy_static;
use num::BigInt;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;

use crate::{io::get_obj, power::Claim};
/// Miner actor method.
pub type Method = fil_actor_miner_state::v8::Method;

pub fn is_v8_miner_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.miner.v8.contains(cid)
}

pub fn is_v9_miner_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.miner.v9.contains(cid)
}

pub fn is_v10_miner_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.miner.v10.contains(cid)
}

pub fn is_v11_miner_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.miner.v11.contains(cid)
}

pub fn is_v12_miner_cid(cid: &Cid) -> bool {
    // The following code cid's existed temporarily on the calibnet testnet, as a "buggy" storage miner actor implementation.
    // See corresponding Lotus PR: https://github.com/filecoin-project/lotus/pull/11363
    lazy_static! {
        static ref V12_POSSIBLE_MINERS: Vec<Cid> = {
            let cids = vec![
                // Calibnet
                "bafk2bzacedwyggrdnk5ofh6fbfw57is2sokduvv77pmtnmhr5ri3nhswvocse", // v12.0.0-dev.0
                "bafk2bzacea7u5i6mqmruz25alik636jpbr4ln5ap5sdlwlz5jslq2ooj2bc3m", // v12.0.0-dev.1
                "bafk2bzacedes62nulops6p46n7uys4wsidss3nne4wt3m37l5jnu2jfgbttww", // v12.0.0-dev.2
                "bafk2bzaceawgt57i7ixpp2bnrek2ewa5g7h5jx3plnsyvho45zfrxrulo4gsk", // Release v12.0.0-dev.3
                "bafk2bzacecnh2ouohmonvebq7uughh4h3ppmg4cjsk74dzxlbbtlcij4xbzxq", // v12.0.0-rc.1
                "bafk2bzaced7emkbbnrewv5uvrokxpf5tlm4jslu2jsv77ofw2yqdglg657uie", // v12.0.0-rc.2
                "bafk2bzaceb7qzqsi5uyxe4o5iuasi47l2hnznvmqr2eu4pl3qscvarjqlnuxo", // v12.0.0-rc.3
                // Devnet
                "bafk2bzaced7bmvrtqzxfpyffijxedgi6jq7ysgwynfuu3n6hndqewoowmf7yi", // v12.0.0-dev.0
                "bafk2bzaceanlq5n6ndlos66bqpftunzylua7pwoaruvpbik46tiopajd6oziq", // v12.0.0-dev.1
                "bafk2bzacea3encig7orwntdkry64mlillvejonxzuczxj4tamtpi3rcmnbr5m", // v12.0.0-dev.2
                "bafk2bzaced42ufdfvd2xk6zyittektpjpnawas6c5cojnxxyvpvigobbbn7cw", // Release v12.0.0-dev.3
                "bafk2bzaceajgt523lr2sf6cacvzo3goyalljlkaoeehyhxlv57wevkljw2cps", // v12.0.0-rc.1
                "bafk2bzaceckqrzomdnfb35byrhabrmmapxplj66cv3efw7u62qswjaqsuxah4", // v12.0.0-rc.2
                "bafk2bzacecs262232b3awcrilyzpdketeayyqzzwgoavtxilgjvayrz55ovk4", // v12.0.0-rc.3
            ];

            cids.into_iter()
                .filter_map(|s| Cid::from_str(s).ok())
                .collect()
        };
    }
    crate::KNOWN_CIDS.actor.miner.v12.contains(cid) || V12_POSSIBLE_MINERS.contains(cid)
}

/// Miner actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    // V7(fil_actor_miner_v7::State),
    V8(fil_actor_miner_state::v8::State),
    V9(fil_actor_miner_state::v9::State),
    V10(fil_actor_miner_state::v10::State),
    V11(fil_actor_miner_state::v11::State),
    V12(fil_actor_miner_state::v12::State),
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_miner_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_miner_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_miner_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_miner_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_miner_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown miner actor code {}", code))
    }

    pub fn info<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<MinerInfo> {
        match self {
            State::V8(st) => Ok(st.get_info(store)?.into()),
            State::V9(st) => Ok(st.get_info(store)?.into()),
            State::V10(st) => Ok(st.get_info(store)?.into()),
            State::V11(st) => Ok(st.get_info(store)?.into()),
            State::V12(st) => Ok(st.get_info(store)?.into()),
        }
    }

    /// Loads deadlines for a miner's state
    pub fn for_each_deadline<BS: Blockstore>(
        &self,
        policy: &Policy,
        store: &BS,
        mut f: impl FnMut(u64, Deadline) -> Result<(), anyhow::Error>,
    ) -> anyhow::Result<()> {
        match self {
            State::V8(st) => st.load_deadlines(&store)?.for_each(
                &from_policy_v10_to_v9(policy),
                &store,
                |idx, dl| f(idx, Deadline::V8(dl)),
            ),
            State::V9(st) => st.load_deadlines(&store)?.for_each(
                &from_policy_v10_to_v9(policy),
                &store,
                |idx, dl| f(idx, Deadline::V9(dl)),
            ),
            State::V10(st) => st
                .load_deadlines(&store)?
                .for_each(policy, &store, |idx, dl| f(idx, Deadline::V10(dl))),
            State::V11(st) => st.load_deadlines(&store)?.for_each(
                &from_policy_v10_to_v11(policy),
                &store,
                |idx, dl| f(idx, Deadline::V11(dl)),
            ),
            State::V12(st) => st
                .load_deadlines(store)?
                .for_each(store, |idx, dl| f(idx, Deadline::V12(dl))),
        }
    }

    /// Loads deadline at index for a miner's state
    pub fn load_deadline<BS: Blockstore>(
        &self,
        policy: &Policy,
        store: &BS,
        idx: u64,
    ) -> anyhow::Result<Deadline> {
        match self {
            State::V8(st) => Ok(st
                .load_deadlines(store)?
                .load_deadline(&from_policy_v10_to_v9(policy), store, idx)
                .map(Deadline::V8)?),
            State::V9(st) => Ok(st
                .load_deadlines(store)?
                .load_deadline(&from_policy_v10_to_v9(policy), store, idx)
                .map(Deadline::V9)?),
            State::V10(st) => Ok(st
                .load_deadlines(store)?
                .load_deadline(policy, store, idx)
                .map(Deadline::V10)?),
            State::V11(st) => Ok(st
                .load_deadlines(store)?
                .load_deadline(&from_policy_v10_to_v11(policy), store, idx)
                .map(Deadline::V11)?),
            State::V12(st) => Ok(st
                .load_deadlines(store)?
                .load_deadline(store, idx)
                .map(Deadline::V12)?),
        }
    }

    /// Loads sectors corresponding to the bitfield. If no bitfield is passed
    /// in, return all.
    pub fn load_sectors<BS: Blockstore>(
        &self,
        store: &BS,
        sectors: Option<&BitField>,
    ) -> anyhow::Result<Vec<SectorOnChainInfo>> {
        match self {
            State::V8(st) => {
                if let Some(sectors) = sectors {
                    Ok(st
                        .load_sector_infos(&store, sectors)?
                        .into_iter()
                        .map(From::from)
                        .collect())
                } else {
                    let sectors = fil_actor_miner_state::v8::Sectors::load(&store, &st.sectors)?;
                    let mut infos = Vec::with_capacity(sectors.amt.count() as usize);
                    sectors.amt.for_each(|_, info| {
                        infos.push(SectorOnChainInfo::from(info.clone()));
                        Ok(())
                    })?;
                    Ok(infos)
                }
            }
            State::V9(st) => {
                if let Some(sectors) = sectors {
                    Ok(st
                        .load_sector_infos(&store, sectors)?
                        .into_iter()
                        .map(From::from)
                        .collect())
                } else {
                    let sectors = fil_actor_miner_state::v9::Sectors::load(&store, &st.sectors)?;
                    let mut infos = Vec::with_capacity(sectors.amt.count() as usize);
                    sectors.amt.for_each(|_, info| {
                        infos.push(SectorOnChainInfo::from(info.clone()));
                        Ok(())
                    })?;
                    Ok(infos)
                }
            }
            State::V10(st) => {
                if let Some(sectors) = sectors {
                    Ok(st
                        .load_sector_infos(&store, sectors)?
                        .into_iter()
                        .map(From::from)
                        .collect())
                } else {
                    let sectors = fil_actor_miner_state::v10::Sectors::load(&store, &st.sectors)?;
                    let mut infos = Vec::with_capacity(sectors.amt.count() as usize);
                    sectors.amt.for_each(|_, info| {
                        infos.push(SectorOnChainInfo::from(info.clone()));
                        Ok(())
                    })?;
                    Ok(infos)
                }
            }
            State::V11(st) => {
                if let Some(sectors) = sectors {
                    Ok(st
                        .load_sector_infos(&store, sectors)?
                        .into_iter()
                        .map(From::from)
                        .collect())
                } else {
                    let sectors = fil_actor_miner_state::v11::Sectors::load(&store, &st.sectors)?;
                    let mut infos = Vec::with_capacity(sectors.amt.count() as usize);
                    sectors.amt.for_each(|_, info| {
                        infos.push(SectorOnChainInfo::from(info.clone()));
                        Ok(())
                    })?;
                    Ok(infos)
                }
            }
            State::V12(st) => {
                if let Some(sectors) = sectors {
                    Ok(st
                        .load_sector_infos(&store, sectors)?
                        .into_iter()
                        .map(From::from)
                        .collect())
                } else {
                    let sectors = fil_actor_miner_state::v12::Sectors::load(&store, &st.sectors)?;
                    let mut infos = Vec::with_capacity(sectors.amt.count() as usize);
                    sectors.amt.for_each(|_, info| {
                        infos.push(SectorOnChainInfo::from(info.clone()));
                        Ok(())
                    })?;
                    Ok(infos)
                }
            }
        }
    }

    /// Gets fee debt of miner state
    pub fn fee_debt(&self) -> TokenAmount {
        match self {
            State::V8(st) => st.fee_debt.clone(),
            State::V9(st) => st.fee_debt.clone(),
            State::V10(st) => from_token_v3_to_v2(st.fee_debt.clone()),
            State::V11(st) => from_token_v3_to_v2(st.fee_debt.clone()),
            State::V12(st) => from_token_v4_to_v2(st.fee_debt.clone()),
        }
    }
}

/// Static information about miner
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MinerInfo {
    pub owner: Address,
    pub worker: Address,
    pub new_worker: Option<Address>,
    pub control_addresses: Vec<Address>, // Must all be ID addresses.
    pub worker_change_epoch: ChainEpoch,
    #[serde(with = "serde_bytes")]
    pub peer_id: Vec<u8>,
    pub multiaddrs: Vec<BytesDe>,
    pub window_post_proof_type: RegisteredPoStProof,
    pub sector_size: SectorSize,
    pub window_post_partition_sectors: u64,
    pub consensus_fault_elapsed: ChainEpoch,
}

impl From<fil_actor_miner_state::v8::MinerInfo> for MinerInfo {
    fn from(info: fil_actor_miner_state::v8::MinerInfo) -> Self {
        MinerInfo {
            owner: info.owner,
            worker: info.worker,
            control_addresses: info
                .control_addresses
                .into_iter()
                .map(Address::from)
                .collect(),
            new_worker: info.pending_worker_key.as_ref().map(|k| k.new_worker),
            worker_change_epoch: info
                .pending_worker_key
                .map(|k| k.effective_at)
                .unwrap_or(-1),
            peer_id: info.peer_id,
            multiaddrs: info.multi_address,
            window_post_proof_type: info.window_post_proof_type,
            sector_size: info.sector_size,
            window_post_partition_sectors: info.window_post_partition_sectors,
            consensus_fault_elapsed: info.consensus_fault_elapsed,
        }
    }
}

impl From<fil_actor_miner_state::v9::MinerInfo> for MinerInfo {
    fn from(info: fil_actor_miner_state::v9::MinerInfo) -> Self {
        MinerInfo {
            owner: info.owner,
            worker: info.worker,
            control_addresses: info
                .control_addresses
                .into_iter()
                .map(Address::from)
                .collect(),
            new_worker: info.pending_worker_key.as_ref().map(|k| k.new_worker),
            worker_change_epoch: info
                .pending_worker_key
                .map(|k| k.effective_at)
                .unwrap_or(-1),
            peer_id: info.peer_id,
            multiaddrs: info.multi_address,
            window_post_proof_type: info.window_post_proof_type,
            sector_size: info.sector_size,
            window_post_partition_sectors: info.window_post_partition_sectors,
            consensus_fault_elapsed: info.consensus_fault_elapsed,
        }
    }
}

impl From<fil_actor_miner_state::v10::MinerInfo> for MinerInfo {
    fn from(info: fil_actor_miner_state::v10::MinerInfo) -> Self {
        MinerInfo {
            owner: from_address_v3_to_v2(info.owner),
            worker: from_address_v3_to_v2(info.worker),
            control_addresses: info
                .control_addresses
                .into_iter()
                .map(from_address_v3_to_v2)
                .collect(),
            new_worker: info
                .pending_worker_key
                .as_ref()
                .map(|k| from_address_v3_to_v2(k.new_worker)),
            worker_change_epoch: info
                .pending_worker_key
                .map(|k| k.effective_at)
                .unwrap_or(-1),
            peer_id: info.peer_id,
            multiaddrs: info.multi_address,
            window_post_proof_type: from_reg_post_proof_v3_to_v2(info.window_post_proof_type),
            sector_size: from_sector_size_v3_to_v2(info.sector_size),
            window_post_partition_sectors: info.window_post_partition_sectors,
            consensus_fault_elapsed: info.consensus_fault_elapsed,
        }
    }
}

impl From<fil_actor_miner_state::v11::MinerInfo> for MinerInfo {
    fn from(info: fil_actor_miner_state::v11::MinerInfo) -> Self {
        MinerInfo {
            owner: from_address_v3_to_v2(info.owner),
            worker: from_address_v3_to_v2(info.worker),
            control_addresses: info
                .control_addresses
                .into_iter()
                .map(from_address_v3_to_v2)
                .collect(),
            new_worker: info
                .pending_worker_key
                .as_ref()
                .map(|k| from_address_v3_to_v2(k.new_worker)),
            worker_change_epoch: info
                .pending_worker_key
                .map(|k| k.effective_at)
                .unwrap_or(-1),
            peer_id: info.peer_id,
            multiaddrs: info.multi_address,
            window_post_proof_type: from_reg_post_proof_v3_to_v2(info.window_post_proof_type),
            sector_size: from_sector_size_v3_to_v2(info.sector_size),
            window_post_partition_sectors: info.window_post_partition_sectors,
            consensus_fault_elapsed: info.consensus_fault_elapsed,
        }
    }
}

impl From<fil_actor_miner_state::v12::MinerInfo> for MinerInfo {
    fn from(info: fil_actor_miner_state::v12::MinerInfo) -> Self {
        MinerInfo {
            owner: from_address_v4_to_v2(info.owner),
            worker: from_address_v4_to_v2(info.worker),
            control_addresses: info
                .control_addresses
                .into_iter()
                .map(from_address_v4_to_v2)
                .collect(),
            new_worker: info
                .pending_worker_key
                .as_ref()
                .map(|k| from_address_v4_to_v2(k.new_worker)),
            worker_change_epoch: info
                .pending_worker_key
                .map(|k| k.effective_at)
                .unwrap_or(-1),
            peer_id: info.peer_id,
            multiaddrs: info.multi_address,
            window_post_proof_type: from_reg_post_proof_v4_to_v2(info.window_post_proof_type),
            sector_size: from_sector_size_v4_to_v2(info.sector_size),
            window_post_partition_sectors: info.window_post_partition_sectors,
            consensus_fault_elapsed: info.consensus_fault_elapsed,
        }
    }
}

impl MinerInfo {
    pub fn worker(&self) -> Address {
        self.worker
    }

    pub fn sector_size(&self) -> SectorSize {
        self.sector_size
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct MinerPower {
    pub miner_power: Claim,
    pub total_power: Claim,
    pub has_min_power: bool,
}

/// Deadline holds the state for all sectors due at a specific deadline.
pub enum Deadline {
    V8(fil_actor_miner_state::v8::Deadline),
    V9(fil_actor_miner_state::v9::Deadline),
    V10(fil_actor_miner_state::v10::Deadline),
    V11(fil_actor_miner_state::v11::Deadline),
    V12(fil_actor_miner_state::v12::Deadline),
}

impl Deadline {
    /// For each partition of the deadline
    pub fn for_each<BS: Blockstore>(
        &self,
        store: &BS,
        mut f: impl FnMut(u64, Partition) -> Result<(), anyhow::Error>,
    ) -> anyhow::Result<()> {
        match self {
            Deadline::V8(dl) => dl.for_each(&store, |idx, part| {
                f(idx, Partition::V8(Cow::Borrowed(part)))
            }),
            Deadline::V9(dl) => dl.for_each(&store, |idx, part| {
                f(idx, Partition::V9(Cow::Borrowed(part)))
            }),
            Deadline::V10(dl) => dl.for_each(&store, |idx, part| {
                f(idx, Partition::V10(Cow::Borrowed(part)))
            }),
            Deadline::V11(dl) => dl.for_each(&store, |idx, part| {
                f(idx, Partition::V11(Cow::Borrowed(part)))
            }),
            Deadline::V12(dl) => dl.for_each(&store, |idx, part| {
                f(idx, Partition::V12(Cow::Borrowed(part)))
            }),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Partition<'a> {
    // V7(Cow<'a, fil_actor_miner_state::v7::Partition>),
    V8(Cow<'a, fil_actor_miner_state::v8::Partition>),
    V9(Cow<'a, fil_actor_miner_state::v9::Partition>),
    V10(Cow<'a, fil_actor_miner_state::v10::Partition>),
    V11(Cow<'a, fil_actor_miner_state::v11::Partition>),
    V12(Cow<'a, fil_actor_miner_state::v12::Partition>),
}

impl Partition<'_> {
    pub fn all_sectors(&self) -> &BitField {
        match self {
            Partition::V8(dl) => &dl.sectors,
            Partition::V9(dl) => &dl.sectors,
            Partition::V10(dl) => &dl.sectors,
            Partition::V11(dl) => &dl.sectors,
            Partition::V12(dl) => &dl.sectors,
        }
    }
    pub fn faulty_sectors(&self) -> &BitField {
        match self {
            Partition::V8(dl) => &dl.faults,
            Partition::V9(dl) => &dl.faults,
            Partition::V10(dl) => &dl.faults,
            Partition::V11(dl) => &dl.faults,
            Partition::V12(dl) => &dl.faults,
        }
    }
    pub fn live_sectors(&self) -> BitField {
        match self {
            Partition::V8(dl) => dl.live_sectors(),
            Partition::V9(dl) => dl.live_sectors(),
            Partition::V10(dl) => dl.live_sectors(),
            Partition::V11(dl) => dl.live_sectors(),
            Partition::V12(dl) => dl.live_sectors(),
        }
    }
    pub fn active_sectors(&self) -> BitField {
        match self {
            Partition::V8(dl) => dl.active_sectors(),
            Partition::V9(dl) => dl.active_sectors(),
            Partition::V10(dl) => dl.active_sectors(),
            Partition::V11(dl) => dl.active_sectors(),
            Partition::V12(dl) => dl.active_sectors(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
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
    pub deal_weight: BigInt,
    /// Integral of active verified deals over sector lifetime
    pub verified_deal_weight: BigInt,
    /// Pledge collected to commit this sector
    pub initial_pledge: TokenAmount,
    /// Expected one day projection of reward for sector computed at activation
    /// time
    pub expected_day_reward: TokenAmount,
    /// Expected twenty day projection of reward for sector computed at
    /// activation time
    pub expected_storage_pledge: TokenAmount,
}

impl From<fil_actor_miner_state::v8::SectorOnChainInfo> for SectorOnChainInfo {
    fn from(info: fil_actor_miner_state::v8::SectorOnChainInfo) -> Self {
        Self {
            sector_number: info.sector_number,
            seal_proof: info.seal_proof,
            sealed_cid: info.sealed_cid,
            deal_ids: info.deal_ids,
            activation: info.activation,
            expiration: info.expiration,
            deal_weight: info.deal_weight,
            verified_deal_weight: info.verified_deal_weight,
            initial_pledge: info.initial_pledge,
            expected_day_reward: info.expected_day_reward,
            expected_storage_pledge: info.expected_storage_pledge,
        }
    }
}

impl From<fil_actor_miner_state::v9::SectorOnChainInfo> for SectorOnChainInfo {
    fn from(info: fil_actor_miner_state::v9::SectorOnChainInfo) -> Self {
        Self {
            sector_number: info.sector_number,
            seal_proof: info.seal_proof,
            sealed_cid: info.sealed_cid,
            deal_ids: info.deal_ids,
            activation: info.activation,
            expiration: info.expiration,
            deal_weight: info.deal_weight,
            verified_deal_weight: info.verified_deal_weight,
            initial_pledge: info.initial_pledge,
            expected_day_reward: info.expected_day_reward,
            expected_storage_pledge: info.expected_storage_pledge,
        }
    }
}

impl From<fil_actor_miner_state::v10::SectorOnChainInfo> for SectorOnChainInfo {
    fn from(info: fil_actor_miner_state::v10::SectorOnChainInfo) -> Self {
        Self {
            sector_number: info.sector_number,
            seal_proof: from_reg_seal_proof_v3_to_v2(info.seal_proof),
            sealed_cid: info.sealed_cid,
            deal_ids: info.deal_ids,
            activation: info.activation,
            expiration: info.expiration,
            deal_weight: info.deal_weight,
            verified_deal_weight: info.verified_deal_weight,
            initial_pledge: from_token_v3_to_v2(info.initial_pledge),
            expected_day_reward: from_token_v3_to_v2(info.expected_day_reward),
            expected_storage_pledge: from_token_v3_to_v2(info.expected_storage_pledge),
        }
    }
}

impl From<fil_actor_miner_state::v11::SectorOnChainInfo> for SectorOnChainInfo {
    fn from(info: fil_actor_miner_state::v11::SectorOnChainInfo) -> Self {
        Self {
            sector_number: info.sector_number,
            seal_proof: from_reg_seal_proof_v3_to_v2(info.seal_proof),
            sealed_cid: info.sealed_cid,
            deal_ids: info.deal_ids,
            activation: info.activation,
            expiration: info.expiration,
            deal_weight: info.deal_weight,
            verified_deal_weight: info.verified_deal_weight,
            initial_pledge: from_token_v3_to_v2(info.initial_pledge),
            expected_day_reward: from_token_v3_to_v2(info.expected_day_reward),
            expected_storage_pledge: from_token_v3_to_v2(info.expected_storage_pledge),
        }
    }
}

impl From<fil_actor_miner_state::v12::SectorOnChainInfo> for SectorOnChainInfo {
    fn from(info: fil_actor_miner_state::v12::SectorOnChainInfo) -> Self {
        Self {
            sector_number: info.sector_number,
            seal_proof: from_reg_seal_proof_v4_to_v2(info.seal_proof),
            sealed_cid: info.sealed_cid,
            deal_ids: info.deal_ids,
            activation: info.activation,
            expiration: info.expiration,
            deal_weight: info.deal_weight,
            verified_deal_weight: info.verified_deal_weight,
            initial_pledge: from_token_v4_to_v2(info.initial_pledge),
            expected_day_reward: from_token_v4_to_v2(info.expected_day_reward),
            expected_storage_pledge: from_token_v4_to_v2(info.expected_storage_pledge),
        }
    }
}
