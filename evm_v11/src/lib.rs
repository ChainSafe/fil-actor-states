use fvm_shared3::error::ExitCode;

use fvm_shared3::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

mod state;
mod types;

pub use state::*;
pub use types::*;

pub const EVM_CONTRACT_REVERTED: ExitCode = ExitCode::new(33);
pub const EVM_CONTRACT_INVALID_INSTRUCTION: ExitCode = ExitCode::new(34);
pub const EVM_CONTRACT_UNDEFINED_INSTRUCTION: ExitCode = ExitCode::new(35);
pub const EVM_CONTRACT_STACK_UNDERFLOW: ExitCode = ExitCode::new(36);
pub const EVM_CONTRACT_STACK_OVERFLOW: ExitCode = ExitCode::new(37);
pub const EVM_CONTRACT_ILLEGAL_MEMORY_ACCESS: ExitCode = ExitCode::new(38);
pub const EVM_CONTRACT_BAD_JUMPDEST: ExitCode = ExitCode::new(39);
pub const EVM_CONTRACT_SELFDESTRUCT_FAILED: ExitCode = ExitCode::new(40);

pub const NATIVE_METHOD_SIGNATURE: &str = "handle_filecoin_method(uint64,uint64,bytes)";
pub const NATIVE_METHOD_SELECTOR: [u8; 4] = [0x86, 0x8e, 0x10, 0xc4];

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
