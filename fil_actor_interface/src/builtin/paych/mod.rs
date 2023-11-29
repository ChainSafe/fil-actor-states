// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;

pub fn is_v10_paych_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.paymentchannel.v10.contains(cid)
}

pub fn is_v11_paych_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.paymentchannel.v11.contains(cid)
}

pub fn is_v12_paych_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS.actor.paymentchannel.v12.contains(cid)
}
