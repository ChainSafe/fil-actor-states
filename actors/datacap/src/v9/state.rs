// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use frc46_token::token;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub governor: Address,
    pub token: token::state::TokenState,
}
