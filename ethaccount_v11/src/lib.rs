use fvm_shared3::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

/// Ethereum Account actor methods.
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
}
