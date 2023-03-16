// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::marker::PhantomData;

use anyhow::Context;
use cid::Cid;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount, piece::PaddedPieceSize};
use num::BigInt;
use serde::Serialize;

use crate::io::get_obj;

/// Market actor address.
pub const ADDRESS: Address = Address::new_id(5);

/// Market actor method.
pub type Method = fil_actor_market_v8::Method;

pub fn is_v8_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.market.v8.contains(cid)
}

pub fn is_v9_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.market.v9.contains(cid)
}

pub fn is_v10_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.market.v10.contains(cid)
}

/// Market actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_market_v8::State),
    V9(fil_actor_market_v9::State),
    V10(fil_actor_market_v10::State),
}

impl State {
    pub fn load<BS>(store: &BS, actor: &ActorState) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_market_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_market_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_market_cid(&actor.code) {
            return get_obj(store, &actor.state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown market actor code {}", actor.code))
    }

    /// Loads escrow table
    pub fn escrow_table<'bs, BS>(&self, _store: &'bs BS) -> anyhow::Result<BalanceTable<'bs, BS>>
    where
        BS: Blockstore,
    {
        unimplemented!()
    }

    /// Loads locked funds table
    pub fn locked_table<'bs, BS>(&self, _store: &'bs BS) -> anyhow::Result<BalanceTable<'bs, BS>>
    where
        BS: Blockstore,
    {
        unimplemented!()
    }

    /// Deal proposals
    pub fn proposals<'bs, BS>(&self, _store: &'bs BS) -> anyhow::Result<DealProposals<'bs, BS>>
    where
        BS: Blockstore,
    {
        unimplemented!()
    }

    /// Deal proposal meta data.
    pub fn states<'bs, BS>(&self, _store: &'bs BS) -> anyhow::Result<DealStates<'bs, BS>>
    where
        BS: Blockstore,
    {
        unimplemented!()
    }

    /// Consume state to return just total funds locked
    pub fn total_locked(&self) -> TokenAmount {
        match self {
            State::V8(st) => st.total_locked(),
            State::V9(st) => st.total_locked(),
            State::V10(st) => st.get_total_locked(),
        }
    }

    /// Validates a collection of deal `dealProposals` for activation, and
    /// returns their combined weight, split into regular deal weight and
    /// verified deal weight.
    pub fn verify_deals_for_activation<BS>(
        &self,
        _store: &BS,
        _deal_ids: &[u64],
        _miner_addr: &Address,
        _sector_expiry: ChainEpoch,
        _curr_epoch: ChainEpoch,
    ) -> anyhow::Result<(BigInt, BigInt)>
    where
        BS: Blockstore,
    {
        unimplemented!()
    }
}

pub enum BalanceTable<'a, BS> {
    UnusedBalanceTable(PhantomData<&'a BS>),
}

pub enum DealProposals<'a, BS> {
    UnusedDealProposal(PhantomData<&'a BS>),
}

impl<BS> DealProposals<'_, BS> {
    pub fn for_each(
        &self,
        _f: impl FnMut(u64, DealProposal) -> anyhow::Result<(), anyhow::Error>,
    ) -> anyhow::Result<()>
    where
        BS: Blockstore,
    {
        unimplemented!()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DealProposal {
    #[serde(rename = "PieceCID")]
    pub piece_cid: Cid,
    pub piece_size: PaddedPieceSize,
    pub verified_deal: bool,
    pub client: Address,
    pub provider: Address,
    // ! This is the field that requires unsafe unchecked utf8 deserialization
    pub label: String,
    pub start_epoch: ChainEpoch,
    pub end_epoch: ChainEpoch,
    pub storage_price_per_epoch: TokenAmount,
    pub provider_collateral: TokenAmount,
    pub client_collateral: TokenAmount,
}

pub enum DealStates<'a, BS> {
    DealStates(PhantomData<&'a BS>),
}

impl<BS> DealStates<'_, BS>
where
    BS: Blockstore,
{
    pub fn get(&self, _key: u64) -> anyhow::Result<Option<DealState>> {
        unimplemented!()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DealState {
    pub sector_start_epoch: ChainEpoch, // -1 if not yet included in proven sector
    pub last_updated_epoch: ChainEpoch, // -1 if deal state never updated
    pub slash_epoch: ChainEpoch,        // -1 if deal never slashed
}

impl<BS> BalanceTable<'_, BS>
where
    BS: Blockstore,
{
    pub fn get(&self, _key: &Address) -> anyhow::Result<TokenAmount> {
        unimplemented!()
    }
}
