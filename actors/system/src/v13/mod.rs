// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use fil_actors_shared::v13::{ActorError, AsActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::CborStore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared4::METHOD_CONSTRUCTOR;
use fvm_shared4::error::ExitCode;
use multihash_codetable::Code;
use num_derive::FromPrimitive;

/// System actor methods.
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
}

/// System actor state.
#[derive(Default, Deserialize_tuple, Serialize_tuple, Debug, Clone)]
pub struct State {
    // builtin actor registry: Vec<(String, Cid)>
    pub builtin_actors: Cid,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let c = store
            .put_cbor(&Vec::<(String, Cid)>::new(), Code::Blake2b256)
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to store system state")?;
        Ok(Self { builtin_actors: c })
    }

    pub fn get_builtin_actors<B: Blockstore>(
        &self,
        store: &B,
    ) -> Result<Vec<(String, Cid)>, String> {
        match store.get_cbor(&self.builtin_actors) {
            Ok(Some(obj)) => Ok(obj),
            Ok(None) => Err("failed to load builtin actor registry; not found".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}
