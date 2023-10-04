// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_shared::frc46_token;
use fil_actors_shared::v12::{ActorError, AsActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared3::address::Address;
use fvm_shared3::error::ExitCode;

#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub struct State {
    pub governor: Address,
    pub token: frc46_token::TokenState,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS, governor: Address) -> Result<State, ActorError> {
        let token_state = frc46_token::TokenState::new(store)
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to create token state")?;
        Ok(State {
            governor,
            token: token_state,
        })
    }
}
