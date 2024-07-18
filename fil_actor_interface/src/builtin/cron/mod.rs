// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use serde::Serialize;

/// Cron actor address.
pub const ADDRESS: Address = Address::new_id(3);

/// Cron actor method.
pub type Method = fil_actor_cron_state::v8::Method;

/// Cron actor state.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum State {
    V8(fil_actor_cron_state::v8::State),
    V9(fil_actor_cron_state::v9::State),
    V10(fil_actor_cron_state::v10::State),
    V11(fil_actor_cron_state::v11::State),
    V12(fil_actor_cron_state::v12::State),
    V13(fil_actor_cron_state::v13::State),
    V14(fil_actor_cron_state::v14::State),
}
