use fil_actors_evm_shared::address::EthAddress;
use fil_actors_runtime::{
    ActorError, AsActorError, EAM_ACTOR_ADDR, INIT_ACTOR_ADDR, WithCodec,
    actor_dispatch_unrestricted, actor_error,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_encoding::{BytesSer, DAG_CBOR};
use fvm_shared4::address::Address;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;

use crate::interpreter::Outcome;
use crate::interpreter::{Bytecode, ExecutionState, System, execute};
use crate::reader::ValueReader;
use cid::Cid;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fvm_shared4::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use types::*;
mod state;
mod types;

pub use state::*;

pub const EVM_CONTRACT_REVERTED: ExitCode = ExitCode::new(33);
pub const EVM_CONTRACT_INVALID_INSTRUCTION: ExitCode = ExitCode::new(34);
pub const EVM_CONTRACT_UNDEFINED_INSTRUCTION: ExitCode = ExitCode::new(35);
pub const EVM_CONTRACT_STACK_UNDERFLOW: ExitCode = ExitCode::new(36);
pub const EVM_CONTRACT_STACK_OVERFLOW: ExitCode = ExitCode::new(37);
pub const EVM_CONTRACT_ILLEGAL_MEMORY_ACCESS: ExitCode = ExitCode::new(38);
pub const EVM_CONTRACT_BAD_JUMPDEST: ExitCode = ExitCode::new(39);
pub const EVM_CONTRACT_SELFDESTRUCT_FAILED: ExitCode = ExitCode::new(40);

const EVM_MAX_RESERVED_METHOD: u64 = 1023;
pub const NATIVE_METHOD_SIGNATURE: &str = "handle_filecoin_method(uint64,uint64,bytes)";
pub const NATIVE_METHOD_SELECTOR: [u8; 4] = [0x86, 0x8e, 0x10, 0xc4];

const EVM_WORD_SIZE: usize = 32;

#[test]
fn test_method_selector() {
    // We could just _generate_ this method selector with a proc macro, but this is easier.
    use multihash_codetable::MultihashDigest;
    let hash = multihash_codetable::Code::Keccak256.digest(NATIVE_METHOD_SIGNATURE.as_bytes());
    let computed_selector = &hash.digest()[..4];
    assert_eq!(computed_selector, NATIVE_METHOD_SELECTOR);
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Resurrect = 2,
    GetBytecode = 3,
    GetBytecodeHash = 4,
    GetStorageAt = 5,
    InvokeContractDelegate = 6,
    InvokeContract = frc42_dispatch::method_hash!("InvokeEVM"),
}
