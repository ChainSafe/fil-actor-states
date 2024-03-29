// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared4::address::Address;

/// State includes the address for the actor
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    pub address: Address,
}
