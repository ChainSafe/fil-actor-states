use fil_actors_shared::v9::{ActorError, AsActorError};
use frc46_token::token;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;

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
