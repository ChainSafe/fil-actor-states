// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::frc46_token;
use fvm_ipld_encoding::tuple::*;
use fvm_shared4::address::Address;

#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub struct State {
    pub governor: Address,
    pub token: frc46_token::TokenState,
}
