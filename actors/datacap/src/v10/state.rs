// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared3::address::Address;

use frc46_token::token;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub governor: Address,
    pub token: token::state::TokenState,
}
