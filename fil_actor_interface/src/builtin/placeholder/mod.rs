// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;

pub fn is_v10_placeholder_cid(cid: &Cid) -> bool {
    crate::KNOWN_CIDS
        .actor
        .placeholder
        .v10
        .as_ref()
        .map_or(false, |cids| cids.contains(cid))
}
