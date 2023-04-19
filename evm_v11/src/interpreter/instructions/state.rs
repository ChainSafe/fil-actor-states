use fil_actors_evm_shared::{address::EthAddress, uints::U256};
use fil_actors_runtime_v11::ActorError;
use fvm_shared::address::Address;

use {
    crate::interpreter::{ExecutionState, System},
    fil_actors_runtime_v11::runtime::Runtime,
};

#[inline]
pub fn balance(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
    actor: U256,
) -> Result<U256, ActorError> {
    let addr: EthAddress = actor.into();
    let addr: Address = addr.into();

    let balance = system
        .rt
        .resolve_address(&addr)
        .and_then(|id| system.rt.actor_balance(id).as_ref().map(U256::from))
        .unwrap_or_default();

    Ok(balance)
}

#[inline]
pub fn selfbalance(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    // Returns native FIL balance of the receiver. Value precision is identical to Ethereum, so
    // no conversion needed (atto, 1e18).
    Ok(U256::from(&system.rt.current_balance()))
}
