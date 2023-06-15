// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_shared::address::Address;

/// verifreg actor address.
pub const ADDRESS: Address = Address::new_id(6);

pub fn is_v8_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v8.contains(cid)
}

pub fn is_v9_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v9.contains(cid)
}

pub fn is_v10_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v10.contains(cid)
}

pub fn is_v11_verifreg_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.verifreg.v11.contains(cid)
}
