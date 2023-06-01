// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use fvm_shared::ActorID;

/// Define Builtin Actor Addresses
pub const SYSTEM_ACTOR_ADDR: Address = Address::new_id(0);
pub const INIT_ACTOR_ADDR: Address = Address::new_id(1);
pub const REWARD_ACTOR_ADDR: Address = Address::new_id(2);
pub const CRON_ACTOR_ADDR: Address = Address::new_id(3);
pub const STORAGE_POWER_ACTOR_ADDR: Address = Address::new_id(4);
pub const STORAGE_MARKET_ACTOR_ADDR: Address = Address::new_id(5);
pub const VERIFIED_REGISTRY_ACTOR_ADDR: Address = Address::new_id(6);
pub const CHAOS_ACTOR_ADDR: Address = Address::new_id(98);
pub const BURNT_FUNDS_ACTOR_ADDR: Address = Address::new_id(99);

/// Defines first available ID address after builtin actors
pub const FIRST_NON_SINGLETON_ADDR: ActorID = 100;
