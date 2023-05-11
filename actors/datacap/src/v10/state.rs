// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use frc46_token::token;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared3::address::Address;
use fvm_shared3::error::ExitCode;

use fil_actors_runtime_v10::{ActorError, AsActorError};

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub governor: Address,
    pub token: token::state::TokenState,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS, governor: Address) -> Result<State, ActorError> {
        let token_state = token::state::TokenState::new(store)
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to create token state")?;
        Ok(State {
            governor,
            token: token_state,
        })
    }
}
