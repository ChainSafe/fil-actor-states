// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use cid::Cid;
use fil_actor_datacap_state::v12::DATACAP_GRANULARITY;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::{Address, Payload};
use num::traits::Euclid;
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
    V13(fil_actor_datacap_state::v13::State),
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

pub fn is_v13_datacap_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.datacap.v13.contains(cid)
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
        if is_v13_datacap_cid(&code) {
            return get_obj(store, &state)?
                .map(State::V13)
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
        let id = match addr.payload() {
            Payload::ID(id) => Ok(*id),
            _ => Err(anyhow!("can only look up ID addresses")),
        }?;

        let int = match self {
            State::V9(state) => state.token.get_balance(store, id),
            State::V11(state) => state.token.get_balance(store, id),
            State::V12(state) => state.token.get_balance(store, id),
            State::V13(state) => state
                .token
                .get_balance(store, id)
                .map(|balance| Some(crate::convert::from_token_v4_to_v3(balance)))
                .map_err(|e| e.to_string().into()),
            _ => return Err(anyhow!("not supported in actors > v8")),
        }?;
        Ok(int
            .map(|amount| amount.atto().to_owned())
            .map(|opt| opt.div_euclid(&BigInt::from(DATACAP_GRANULARITY))))
    }
}
