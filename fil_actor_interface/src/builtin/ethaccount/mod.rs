// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;

pub fn is_v10_ethaccount_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.ethaccount.v10.contains(cid)
}

pub fn is_v11_ethaccount_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.ethaccount.v11.contains(cid)
}

pub fn is_v12_ethaccount_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.ethaccount.v12.contains(cid)
}

pub fn is_v13_ethaccount_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.ethaccount.v13.contains(cid)
}

pub fn is_v14_ethaccount_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.ethaccount.v14.contains(cid)
}
