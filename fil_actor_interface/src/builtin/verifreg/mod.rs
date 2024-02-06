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
use num::BigInt;
use serde::Serialize;

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
    crate::KNOWN_CIDS.actor.verifreg.v13.contains(cid)
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
}
