use fil_actors_evm_shared_v11::uints::U256;
use fil_actors_runtime_v11::ActorError;
use fvm_shared::clock::ChainEpoch;

use crate::EVM_WORD_SIZE;

use {
    crate::interpreter::{ExecutionState, System},
    fil_actors_runtime_v11::runtime::Runtime,
};

#[inline]
pub fn blockhash(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
    bn: U256,
) -> Result<U256, ActorError> {
    let result = bn
        .try_into()
        .ok()
        .filter(|&height: &ChainEpoch| {
            // The EVM allows fetching blockhashes from the 256 _previous_ blocks, not including the
            // current. The FVM allows fetching block CIDs from the last 899 epochs, not including
            // the current epoch.
            let curr_epoch = system.rt.curr_epoch();
            height >= curr_epoch - 256 && height < curr_epoch
        })
        .and_then(|height| system.rt.tipset_cid(height).ok())
        .map(|cid| {
            let mut hash = cid.hash().digest();
            if hash.len() > EVM_WORD_SIZE {
                hash = &hash[..EVM_WORD_SIZE]
            }
            U256::from_big_endian(hash)
        })
        .unwrap_or_default();
    Ok(result)
}

#[inline]
pub fn caller(state: &mut ExecutionState, _: &System<impl Runtime>) -> Result<U256, ActorError> {
    Ok(state.caller.as_evm_word())
}

#[inline]
pub fn address(
    state: &mut ExecutionState,
    _system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    Ok(state.receiver.as_evm_word())
}

#[inline]
pub fn origin(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    let origin_addr = system
        .resolve_ethereum_address(&system.rt.message().origin())
        .expect("failed to resolve origin address");
    Ok(origin_addr.as_evm_word())
}

#[inline]
pub fn call_value(
    state: &mut ExecutionState,
    _system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    Ok(U256::from(&state.value_received))
}

#[inline]
pub fn coinbase(
    _state: &mut ExecutionState,
    _system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    // Eth zero address, beneficiary of the current block doesn't make much sense in Filecoin due to multiple winners in each block.
    Ok(U256::zero())
}

#[inline]
pub fn gas_price(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    let effective_price = system.rt.base_fee() + system.rt.message().gas_premium();
    Ok(U256::from(&effective_price))
}

#[inline]
pub fn gas(_state: &mut ExecutionState, system: &System<impl Runtime>) -> Result<U256, ActorError> {
    Ok(U256::from(system.rt.gas_available()))
}

#[inline]
pub fn timestamp(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    Ok(U256::from(system.rt.tipset_timestamp()))
}

#[inline]
pub fn block_number(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    // NOTE: Panics if current epoch is negative, which should never happen in the network
    Ok(U256::from(system.rt.curr_epoch()))
}

/// EIP-4399: DIFFICULTY -> PREVRANDAO
#[inline]
pub fn prevrandao(
    _state: &mut ExecutionState,
    system: &mut System<impl Runtime>,
) -> Result<U256, ActorError> {
    // NOTE: Filecoin beacon randomness is expected to fall outside of the `2^64` reserved range, following PREVRANDAO's assumptions.
    // NOTE: EVM uses previous RANDAO value in this opcode since the _current_ RANDAO for them runs on the beacon chain's state
    //      and wont be finalized till the end of a block. Filecoin's chain randomness is generated _before_ any contract is run, so we instead
    //      grab randomness from the current epoch.
    system.get_randomness().map(|v| U256::from(*v))
}

#[inline]
pub fn gas_limit(
    _state: &mut ExecutionState,
    _system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    const BLOCK_GAS_LIMIT: u64 = 10_000_000_000u64;
    Ok(U256::from(BLOCK_GAS_LIMIT))
}

#[inline]
pub fn chain_id(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    Ok(U256::from_u64(system.rt.chain_id().into()))
}

#[inline]
pub fn base_fee(
    _state: &mut ExecutionState,
    system: &System<impl Runtime>,
) -> Result<U256, ActorError> {
    Ok(U256::from(&system.rt.base_fee()))
}
