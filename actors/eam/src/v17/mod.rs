use std::iter;

use fil_actors_evm_shared::address::EthAddress;
use num_traits::Zero;

use ext::{
    account::PUBKEY_ADDRESS_METHOD,
    evm::RESURRECT_METHOD,
    init::{Exec4Params, Exec4Return},
};
use fil_actors_runtime::{
    ActorError, AsActorError, EAM_ACTOR_ID, INIT_ACTOR_ADDR, SYSTEM_ACTOR_ADDR,
    actor_dispatch_unrestricted, actor_error, deserialize_block, extract_send_result,
};

use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{ActorID, METHOD_CONSTRUCTOR, error::ExitCode, sys::SendFlags};
use serde::{Deserialize, Serialize};

use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Runtime};

use fvm_ipld_encoding::{RawBytes, strict_bytes, tuple::*};
use fvm_shared::address::{Address, Payload};
use fvm_shared::crypto::hash::SupportedHashes;
use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Create = 2,
    Create2 = 3,
    CreateExternal = 4,
}

/// Compute the a new actor address using the EVM's CREATE rules.
pub fn compute_address_create(rt: &impl Runtime, from: &EthAddress, nonce: u64) -> EthAddress {
    let mut stream = rlp::RlpStream::new();
    stream.begin_list(2).append(&&from.0[..]).append(&nonce);
    EthAddress(hash_20(rt, &stream.out()))
}

/// Compute the a new actor address using the EVM's CREATE2 rules.
pub fn compute_address_create2(
    rt: &impl Runtime,
    from: &EthAddress,
    salt: &[u8; 32],
    initcode: &[u8],
) -> EthAddress {
    let inithash = rt.hash(SupportedHashes::Keccak256, initcode);
    EthAddress(hash_20(
        rt,
        &[&[0xff], &from.0[..], salt, &inithash].concat(),
    ))
}

pub fn compute_address_create_external(rt: &impl Runtime, from: &EthAddress) -> EthAddress {
    compute_address_create(rt, from, rt.message().nonce())
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct CreateParams {
    #[serde(with = "strict_bytes")]
    pub initcode: Vec<u8>,
    pub nonce: u64,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct Create2Params {
    #[serde(with = "strict_bytes")]
    pub initcode: Vec<u8>,
    #[serde(with = "strict_bytes")]
    pub salt: [u8; 32],
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct CreateExternalParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

#[derive(Serialize_tuple, Deserialize_tuple, Debug, PartialEq, Eq)]
pub struct Return {
    pub actor_id: ActorID,
    pub robust_address: Option<Address>,
    pub eth_address: EthAddress,
}

pub type CreateReturn = Return;
pub type Create2Return = Return;
pub type CreateExternalReturn = Return;
