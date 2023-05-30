// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::ActorID;

/// Defines first available ID address after builtin actors
pub const FIRST_NON_SINGLETON_ADDR: ActorID = 100;

/// State is reponsible for creating
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub address_map: Cid,
    pub next_id: ActorID,
    pub network_name: String,
}

impl State {
    pub fn new(address_map: Cid, network_name: String) -> Self {
        Self {
            address_map,
            next_id: FIRST_NON_SINGLETON_ADDR,
            network_name,
        }
    }
}
