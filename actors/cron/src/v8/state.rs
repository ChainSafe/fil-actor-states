// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::MethodNum;
use fvm_shared::address::Address;

/// Cron actor state which holds entries to call during epoch tick
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// Entries is a set of actors (and corresponding methods) to call during `EpochTick`.
    pub entries: Vec<Entry>,
}

#[derive(Clone, PartialEq, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Entry {
    /// The actor to call (ID address)
    pub receiver: Address,
    /// The method number to call (must accept empty parameters)
    pub method_num: MethodNum,
}
