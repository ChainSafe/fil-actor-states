// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;

pub fn is_v10_placeholder_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.placeholder.v10.contains(cid)
}

pub fn is_v11_placeholder_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.placeholder.v11.contains(cid)
}

pub fn is_v12_placeholder_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.placeholder.v12.contains(cid)
}

pub fn is_v13_placeholder_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.placeholder.v13.contains(cid)
}

pub fn is_v14_placeholder_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.placeholder.v14.contains(cid)
}
