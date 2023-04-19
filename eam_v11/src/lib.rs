use fvm_shared::METHOD_CONSTRUCTOR;
pub mod ext;

use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Create = 2,
    Create2 = 3,
    CreateExternal = 4,
}
