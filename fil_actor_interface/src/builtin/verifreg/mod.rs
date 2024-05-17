// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::io::get_obj;
use anyhow::{anyhow, Context};
use cid::Cid;
use fil_actors_shared::v8::{make_map_with_root_and_bitwidth, HAMT_BIT_WIDTH};
use fil_actors_shared::v9::Keyer;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::{Address, Protocol};
use fvm_shared4::bigint::bigint_ser::BigIntDe;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::piece::PaddedPieceSize;
use fvm_shared4::ActorID;
use num::BigInt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// verifreg actor address.
pub const ADDRESS: Address = Address::new_id(6);

/// Verifreg actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_verifreg_state::v8::State),
    V9(fil_actor_verifreg_state::v9::State),
    V10(fil_actor_verifreg_state::v10::State),
    V11(fil_actor_verifreg_state::v11::State),
    V12(fil_actor_verifreg_state::v12::State),
    V13(fil_actor_verifreg_state::v13::State),
}

pub fn is_v8_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v8.contains(cid)
}

pub fn is_v9_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v9.contains(cid)
}

pub fn is_v10_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v10.contains(cid)
}

pub fn is_v11_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v11.contains(cid)
}

pub fn is_v12_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v12.contains(cid)
}

pub fn is_v13_verifreg_cid(cid: &Cid) -> bool {
    // The following CID existed in the NV22 network, but was fixed as a patch.
    // See corresponding Lotus PR: https://github.com/filecoin-project/lotus/pull/11776
    lazy_static::lazy_static! {
        static ref PATCH_VERIFREG_V13: Cid =
            Cid::from_str("bafk2bzacednskl3bykz5qpo54z2j2p4q44t5of4ktd6vs6ymmg2zebsbxazkm")
                .expect("hardcoded CID must be valid");
    }
    crate::KNOWN_CIDS.actor.verifreg.v13.contains(cid) || cid == &*PATCH_VERIFREG_V13
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v8_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V8)
                .context("Actor state doesn't exist in store");
        }
        if is_v9_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        if is_v13_verifreg_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V13)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown verifreg actor code {}", code))
    }

    pub fn verified_client_data_cap<BS>(
        &self,
        store: &BS,
        addr: Address,
    ) -> anyhow::Result<Option<BigInt>>
    where
        BS: Blockstore,
    {
        if addr.protocol() != Protocol::ID {
            return Err(anyhow!("can only look up ID addresses"));
        }

        match self {
            State::V8(state) => {
                let vh = make_map_with_root_and_bitwidth(
                    &state.verified_clients,
                    store,
                    HAMT_BIT_WIDTH,
                )?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            _ => Err(anyhow!("not supported in actors > v8")),
        }
    }

    pub fn verifier_data_cap<BS>(&self, store: &BS, addr: Address) -> anyhow::Result<Option<BigInt>>
    where
        BS: Blockstore,
    {
        if addr.protocol() != Protocol::ID {
            return Err(anyhow!("can only look up ID addresses"));
        }

        match self {
            State::V8(state) => {
                let vh = make_map_with_root_and_bitwidth(&state.verifiers, store, HAMT_BIT_WIDTH)?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V9(state) => {
                let vh = make_map_with_root_and_bitwidth(&state.verifiers, store, HAMT_BIT_WIDTH)?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V10(state) => {
                let vh = make_map_with_root_and_bitwidth(&state.verifiers, store, HAMT_BIT_WIDTH)?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V11(state) => {
                let vh = make_map_with_root_and_bitwidth(&state.verifiers, store, HAMT_BIT_WIDTH)?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V12(state) => {
                let vh = make_map_with_root_and_bitwidth(&state.verifiers, store, HAMT_BIT_WIDTH)?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V13(state) => {
                let vh = make_map_with_root_and_bitwidth(&state.verifiers, store, HAMT_BIT_WIDTH)?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
        }
    }

    pub fn get_allocation<BS>(
        &self,
        store: &BS,
        addr: ActorID,
        allocation_id: AllocationID,
    ) -> anyhow::Result<Option<Allocation>>
    where
        BS: Blockstore,
    {
        match self {
            State::V8(_) => Err(anyhow!("unsupported in actors v8")),
            State::V9(state) => {
                let mut map = state.load_allocs(store)?;
                Ok(fil_actor_verifreg_state::v9::state::get_allocation(
                    &mut map,
                    addr,
                    allocation_id,
                )?
                .map(Allocation::from))
            }
            State::V10(state) => {
                let mut map = state.load_allocs(store)?;
                Ok(fil_actor_verifreg_state::v10::state::get_allocation(
                    &mut map,
                    addr,
                    allocation_id,
                )?
                .map(Allocation::from))
            }
            State::V11(state) => {
                let mut map = state.load_allocs(store)?;
                Ok(fil_actor_verifreg_state::v11::state::get_allocation(
                    &mut map,
                    addr,
                    allocation_id,
                )?
                .map(Allocation::from))
            }
            State::V12(state) => {
                let mut map = state.load_allocs(store)?;
                Ok(fil_actor_verifreg_state::v12::state::get_allocation(
                    &mut map,
                    addr,
                    allocation_id,
                )?
                .map(Allocation::from))
            }
            State::V13(state) => {
                let mut map = state.load_allocs(store)?;
                Ok(fil_actor_verifreg_state::v13::state::get_allocation(
                    &mut map,
                    addr,
                    allocation_id,
                )?
                .map(Allocation::from))
            }
        }
    }
}

pub type AllocationID = u64;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Allocation {
    // The verified client which allocated the DataCap.
    pub client: ActorID,
    // The provider (miner actor) which may claim the allocation.
    pub provider: ActorID,
    // Identifier of the data to be committed.
    pub data: Cid,
    // The (padded) size of data.
    pub size: PaddedPieceSize,
    // The minimum duration which the provider must commit to storing the piece to avoid
    // early-termination penalties (epochs).
    pub term_min: ChainEpoch,
    // The maximum period for which a provider can earn quality-adjusted power
    // for the piece (epochs).
    pub term_max: ChainEpoch,
    // The latest epoch by which a provider must commit data before the allocation expires.
    pub expiration: ChainEpoch,
}

macro_rules! from_allocation {
    ($type: ty) => {
        impl From<&$type> for Allocation {
            fn from(alloc: &$type) -> Self {
                Self {
                    client: alloc.client,
                    provider: alloc.provider,
                    data: alloc.data,
                    size: PaddedPieceSize(alloc.size.0),
                    term_min: alloc.term_min,
                    term_max: alloc.term_max,
                    expiration: alloc.expiration,
                }
            }
        }
    };
}

from_allocation!(fil_actor_verifreg_state::v13::Allocation);
from_allocation!(fil_actor_verifreg_state::v12::Allocation);
from_allocation!(fil_actor_verifreg_state::v11::Allocation);
from_allocation!(fil_actor_verifreg_state::v10::Allocation);
from_allocation!(fil_actor_verifreg_state::v9::Allocation);
