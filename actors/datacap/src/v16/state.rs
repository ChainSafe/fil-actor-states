use frc46_token::token;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared4::ActorID;
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;

use fil_actors_shared::v16::{ActorError, AsActorError};

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

    // Visible for testing
    pub fn balance<BS: Blockstore>(
        &self,
        bs: &BS,
        owner: ActorID,
    ) -> Result<TokenAmount, ActorError> {
        self.token
            .get_balance(bs, owner)
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to get balance")
    }
}
