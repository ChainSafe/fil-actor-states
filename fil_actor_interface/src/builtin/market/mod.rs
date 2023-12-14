// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{from_token_v3_to_v2, from_token_v4_to_v2};
use anyhow::Context;
use cid::Cid;
use fil_actor_market_state::v10::DealArray as V10DealArray;
use fil_actor_market_state::v10::DealMetaArray as V10DealMetaArray;
use fil_actor_market_state::v11::DealArray as V11DealArray;
use fil_actor_market_state::v11::DealMetaArray as V11DealMetaArray;
use fil_actor_market_state::v12::DealArray as V12DealArray;
use fil_actor_market_state::v12::DealMetaArray as V12DealMetaArray;
use fil_actor_market_state::v9::DealArray as V9DealArray;
use fil_actor_market_state::v9::DealMetaArray as V9DealMetaArray;
use fil_actors_shared::v10::AsActorError as V10AsActorError;
use fil_actors_shared::v11::AsActorError as V11AsActorError;
use fil_actors_shared::v12::AsActorError as V12AsActorError;
use fil_actors_shared::v9::AsActorError as V9AsActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::error::ExitCode as FVMExitCode;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount, piece::PaddedPieceSize};
use fvm_shared3::error::ExitCode as FVM3ExitCode;
use fvm_shared4::error::ExitCode as FVM4ExitCode;
use serde::Serialize;
use std::marker::PhantomData;

use crate::io::get_obj;

/// Market actor address.
pub const ADDRESS: Address = Address::new_id(5);

/// Market actor method.
pub type Method = fil_actor_market_state::v8::Method;

pub fn is_v8_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.market.v8.contains(cid)
}

pub fn is_v9_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.market.v9.contains(cid)
}

pub fn is_v10_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.market.v10.contains(cid)
}

pub fn is_v11_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.market.v11.contains(cid)
}

pub fn is_v12_market_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.market.v12.contains(cid)
}

/// Market actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_market_state::v8::State),
    V9(fil_actor_market_state::v9::State),
    V10(fil_actor_market_state::v10::State),
    V11(fil_actor_market_state::v11::State),
    V12(fil_actor_market_state::v12::State),
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_market_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_market_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_market_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_market_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_market_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown market actor code {}", code))
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
    pub fn proposals<'bs, BS>(&'bs self, store: &'bs BS) -> anyhow::Result<DealProposals<'bs, BS>>
    where
        BS: Blockstore,
    {
        match self {
            // TODO: `get_proposal_array` DNE for V8
            State::V8(_st) => unimplemented!(),
            // TODO: `get_proposal_array` DNE for V9
            State::V9(_st) => unimplemented!(),
            State::V10(st) => Ok(DealProposals::V10(st.get_proposal_array(store)?)),
            State::V11(st) => Ok(DealProposals::V11(st.get_proposal_array(store)?)),
            State::V12(st) => Ok(DealProposals::V12(st.get_proposal_array(store)?)),
        }
    }

    /// Deal proposal meta data.
    pub fn states<'bs, BS>(&self, store: &'bs BS) -> anyhow::Result<DealStates<'bs, BS>>
    where
        BS: Blockstore,
    {
        match self {
            // TODO: `DealMetaArray::load` DNE for V8
            State::V8(_st) => unimplemented!(),
            State::V9(st) => Ok(DealStates::V9(V9AsActorError::context_code(
                V9DealMetaArray::load(&st.states, store),
                FVMExitCode::USR_ILLEGAL_STATE,
                "failed to load deal state array",
            )?)),
            State::V10(st) => Ok(DealStates::V10(V10AsActorError::context_code(
                V10DealMetaArray::load(&st.states, store),
                FVM3ExitCode::USR_ILLEGAL_STATE,
                "failed to load deal state array",
            )?)),
            State::V11(st) => Ok(DealStates::V11(V11AsActorError::context_code(
                V11DealMetaArray::load(&st.states, store),
                FVM3ExitCode::USR_ILLEGAL_STATE,
                "failed to load deal state array",
            )?)),
            State::V12(st) => Ok(DealStates::V12(V12AsActorError::context_code(
                V12DealMetaArray::load(&st.states, store),
                FVM4ExitCode::USR_ILLEGAL_STATE,
                "failed to load deal state array",
            )?)),
        }
    }

    /// Consume state to return just total funds locked
    pub fn total_locked(&self) -> TokenAmount {
        match self {
            State::V8(st) => st.total_locked(),
            State::V9(st) => st.total_locked(),
            State::V10(st) => from_token_v3_to_v2(st.get_total_locked()),
            State::V11(st) => from_token_v3_to_v2(st.get_total_locked()),
            State::V12(st) => from_token_v4_to_v2(st.get_total_locked()),
        }
    }
}

pub enum BalanceTable<'a, BS> {
    UnusedBalanceTable(PhantomData<&'a BS>),
}

pub enum DealProposals<'bs, BS> {
    // TODO: use correct V8 type
    V8(V9DealArray<'bs, BS>),
    V9(V9DealArray<'bs, BS>),
    V10(V10DealArray<'bs, BS>),
    V11(V11DealArray<'bs, BS>),
    V12(V12DealArray<'bs, BS>),
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

pub enum DealStates<'bs, BS> {
    V8(V9DealMetaArray<'bs, BS>),
    V9(V9DealMetaArray<'bs, BS>),
    V10(V10DealMetaArray<'bs, BS>),
    V11(V11DealMetaArray<'bs, BS>),
    V12(V12DealMetaArray<'bs, BS>),
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
