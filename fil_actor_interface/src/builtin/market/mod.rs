// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::convert::{
    from_address_v3_to_v2, from_address_v4_to_v2, from_padded_piece_size_v3_to_v2,
    from_padded_piece_size_v4_to_v2, from_token_v3_to_v2, from_token_v4_to_v2,
};
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
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::io::get_obj;

/// Market actor address.
pub const ADDRESS: Address = Address::new_id(5);

/// Market actor method.
pub type Method = fil_actor_market_state::v8::Method;

pub type AllocationID = u64;

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
            // `get_proposal_array` does not exist for V8
            State::V8(_st) => anyhow::bail!("unimplemented"),
            // `get_proposal_array` does not exist for V9
            State::V9(_st) => anyhow::bail!("unimplemented"),
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
            // `DealMetaArray::load` does not exist for V8
            State::V8(_st) => anyhow::bail!("unimplemented"),
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
    V9(V9DealArray<'bs, BS>),
    V10(V10DealArray<'bs, BS>),
    V11(V11DealArray<'bs, BS>),
    V12(V12DealArray<'bs, BS>),
}

impl<BS> DealProposals<'_, BS> {
    pub fn for_each(
        &self,
        mut f: impl FnMut(u64, DealProposal) -> anyhow::Result<(), anyhow::Error>,
    ) -> anyhow::Result<()>
    where
        BS: Blockstore,
    {
        match self {
            DealProposals::V9(deal_array) => {
                deal_array.for_each(|key, deal_proposal| f(key, deal_proposal.into()))?;
                Ok(())
            }
            DealProposals::V10(deal_array) => {
                deal_array.for_each(|key, deal_proposal| f(key, deal_proposal.into()))?;
                Ok(())
            }
            DealProposals::V11(deal_array) => {
                deal_array.for_each(|key, deal_proposal| f(key, deal_proposal.into()))?;
                Ok(())
            }
            DealProposals::V12(deal_array) => {
                deal_array.for_each(|key, deal_proposal| f(key, deal_proposal.into()))?;
                Ok(())
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
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

impl From<&fil_actor_market_state::v9::DealProposal> for DealProposal {
    fn from(deal_proposal: &fil_actor_market_state::v9::DealProposal) -> Self {
        Self {
            piece_cid: deal_proposal.piece_cid,
            piece_size: deal_proposal.piece_size,
            verified_deal: deal_proposal.verified_deal,
            client: deal_proposal.client,
            provider: deal_proposal.provider,
            label: match &deal_proposal.label {
                fil_actor_market_state::v9::Label::String(s) => s.clone(),
                fil_actor_market_state::v9::Label::Bytes(b) => {
                    String::from_utf8(b.clone()).expect("failed to deserialize utf8 string")
                }
            },
            start_epoch: deal_proposal.start_epoch,
            end_epoch: deal_proposal.end_epoch,
            storage_price_per_epoch: deal_proposal.storage_price_per_epoch.clone(),
            provider_collateral: deal_proposal.provider_collateral.clone(),
            client_collateral: deal_proposal.client_collateral.clone(),
        }
    }
}

impl From<&fil_actor_market_state::v10::DealProposal> for DealProposal {
    fn from(deal_proposal: &fil_actor_market_state::v10::DealProposal) -> Self {
        Self {
            piece_cid: deal_proposal.piece_cid,
            piece_size: from_padded_piece_size_v3_to_v2(deal_proposal.piece_size),
            verified_deal: deal_proposal.verified_deal,
            client: from_address_v3_to_v2(deal_proposal.client),
            provider: from_address_v3_to_v2(deal_proposal.provider),
            label: match &deal_proposal.label {
                fil_actor_market_state::v10::Label::String(s) => s.clone(),
                fil_actor_market_state::v10::Label::Bytes(b) => {
                    String::from_utf8(b.clone()).expect("failed to deserialize utf8 string")
                }
            },
            start_epoch: deal_proposal.start_epoch,
            end_epoch: deal_proposal.end_epoch,
            storage_price_per_epoch: from_token_v3_to_v2(
                deal_proposal.storage_price_per_epoch.clone(),
            ),
            provider_collateral: from_token_v3_to_v2(deal_proposal.provider_collateral.clone()),
            client_collateral: from_token_v3_to_v2(deal_proposal.client_collateral.clone()),
        }
    }
}

impl From<&fil_actor_market_state::v11::DealProposal> for DealProposal {
    fn from(deal_proposal: &fil_actor_market_state::v11::DealProposal) -> Self {
        Self {
            piece_cid: deal_proposal.piece_cid,
            piece_size: from_padded_piece_size_v3_to_v2(deal_proposal.piece_size),
            verified_deal: deal_proposal.verified_deal,
            client: from_address_v3_to_v2(deal_proposal.client),
            provider: from_address_v3_to_v2(deal_proposal.provider),
            label: match &deal_proposal.label {
                fil_actor_market_state::v11::Label::String(s) => s.clone(),
                fil_actor_market_state::v11::Label::Bytes(b) => {
                    String::from_utf8(b.clone()).expect("failed to deserialize utf8 string")
                }
            },
            start_epoch: deal_proposal.start_epoch,
            end_epoch: deal_proposal.end_epoch,
            storage_price_per_epoch: from_token_v3_to_v2(
                deal_proposal.storage_price_per_epoch.clone(),
            ),
            provider_collateral: from_token_v3_to_v2(deal_proposal.provider_collateral.clone()),
            client_collateral: from_token_v3_to_v2(deal_proposal.client_collateral.clone()),
        }
    }
}

impl From<&fil_actor_market_state::v12::DealProposal> for DealProposal {
    fn from(deal_proposal: &fil_actor_market_state::v12::DealProposal) -> Self {
        Self {
            piece_cid: deal_proposal.piece_cid,
            piece_size: from_padded_piece_size_v4_to_v2(deal_proposal.piece_size),
            verified_deal: deal_proposal.verified_deal,
            client: from_address_v4_to_v2(deal_proposal.client),
            provider: from_address_v4_to_v2(deal_proposal.provider),
            label: match &deal_proposal.label {
                fil_actor_market_state::v12::Label::String(s) => s.clone(),
                fil_actor_market_state::v12::Label::Bytes(b) => {
                    String::from_utf8(b.clone()).expect("failed to deserialize utf8 string")
                }
            },
            start_epoch: deal_proposal.start_epoch,
            end_epoch: deal_proposal.end_epoch,
            storage_price_per_epoch: from_token_v4_to_v2(
                deal_proposal.storage_price_per_epoch.clone(),
            ),
            provider_collateral: from_token_v4_to_v2(deal_proposal.provider_collateral.clone()),
            client_collateral: from_token_v4_to_v2(deal_proposal.client_collateral.clone()),
        }
    }
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
    pub fn get(&self, key: u64) -> anyhow::Result<Option<DealState>> {
        match self {
            DealStates::V8(deal_array) => Ok(deal_array.get(key)?.map(|deal_state| DealState {
                sector_start_epoch: deal_state.sector_start_epoch,
                last_updated_epoch: deal_state.last_updated_epoch,
                slash_epoch: deal_state.slash_epoch,
                verified_claim: deal_state.verified_claim,
            })),
            DealStates::V9(deal_array) => Ok(deal_array.get(key)?.map(|deal_state| DealState {
                sector_start_epoch: deal_state.sector_start_epoch,
                last_updated_epoch: deal_state.last_updated_epoch,
                slash_epoch: deal_state.slash_epoch,
                verified_claim: deal_state.verified_claim,
            })),
            DealStates::V10(deal_array) => Ok(deal_array.get(key)?.map(|deal_state| DealState {
                sector_start_epoch: deal_state.sector_start_epoch,
                last_updated_epoch: deal_state.last_updated_epoch,
                slash_epoch: deal_state.slash_epoch,
                verified_claim: deal_state.verified_claim,
            })),
            DealStates::V11(deal_array) => Ok(deal_array.get(key)?.map(|deal_state| DealState {
                sector_start_epoch: deal_state.sector_start_epoch,
                last_updated_epoch: deal_state.last_updated_epoch,
                slash_epoch: deal_state.slash_epoch,
                verified_claim: deal_state.verified_claim,
            })),
            DealStates::V12(deal_array) => Ok(deal_array.get(key)?.map(|deal_state| DealState {
                sector_start_epoch: deal_state.sector_start_epoch,
                last_updated_epoch: deal_state.last_updated_epoch,
                slash_epoch: deal_state.slash_epoch,
                verified_claim: deal_state.verified_claim,
            })),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct DealState {
    pub sector_start_epoch: ChainEpoch, // -1 if not yet included in proven sector
    pub last_updated_epoch: ChainEpoch, // -1 if deal state never updated
    pub slash_epoch: ChainEpoch,        // -1 if deal never slashed
    pub verified_claim: AllocationID, // ID of the verified registry allocation/claim for this deal's data (0 if none).
}

impl<BS> BalanceTable<'_, BS>
where
    BS: Blockstore,
{
    pub fn get(&self, _key: &Address) -> anyhow::Result<TokenAmount> {
        unimplemented!()
    }
}
