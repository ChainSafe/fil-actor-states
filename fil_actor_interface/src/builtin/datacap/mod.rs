// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use cid::Cid;
use fil_actors_shared::v9::Keyer;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared4::bigint::bigint_ser::BigIntDe;
use fvm_shared::address::{Address, Protocol};
use num::BigInt;
use serde::Serialize;

use crate::io::get_obj;

/// Datacap actor method.
pub type Method = fil_actor_datacap_state::v10::Method;

/// Datacap actor address.
pub const ADDRESS: Address = Address::new_id(7);

/// Datacap actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V9(fil_actor_datacap_state::v9::State),
    V10(fil_actor_datacap_state::v10::State),
    V11(fil_actor_datacap_state::v11::State),
    V12(fil_actor_datacap_state::v12::State),
}

pub fn is_v9_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v9.contains(cid)
}

pub fn is_v10_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v10.contains(cid)
}

pub fn is_v11_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v11.contains(cid)
}

pub fn is_v12_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v12.contains(cid)
}

impl State {
    pub fn load<BS>(store: &BS, code: Cid, state: Cid) -> anyhow::Result<State>
    where
        BS: Blockstore,
    {
        if is_v9_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V9)
                .context("Actor state doesn't exist in store");
        }
        if is_v10_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V10)
                .context("Actor state doesn't exist in store");
        }
        if is_v11_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V11)
                .context("Actor state doesn't exist in store");
        }
        if is_v12_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V12)
                .context("Actor state doesn't exist in store");
        }
        Err(anyhow::anyhow!("Unknown datacap actor code {}", code))
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
            State::V9(state) => {
                let vh = fil_actors_shared::v9::make_map_with_root_and_bitwidth(
                    &state.token.balances,
                    store,
                    state.token.hamt_bit_width,
                )?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V11(state) => {
                let vh = fil_actors_shared::v11::make_map_with_root_and_bitwidth(
                    &state.token.balances,
                    store,
                    state.token.hamt_bit_width,
                )?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            State::V12(state) => {
                let vh = fil_actors_shared::v12::make_map_with_root_and_bitwidth(
                    &state.token.balances,
                    store,
                    state.token.hamt_bit_width,
                )?;
                Ok(vh.get(&addr.key())?.map(|int: &BigIntDe| int.0.to_owned()))
            }
            _ => Err(anyhow!("not supported in actors > v8")),
        }
    }
}
