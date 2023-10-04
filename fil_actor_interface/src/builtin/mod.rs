// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod account;
pub mod cron;
pub mod datacap;
pub mod ethaccount;
pub mod evm;
pub mod init;
pub mod known_cids;
pub mod market;
pub mod miner;
pub mod multisig;
pub mod placeholder;
pub mod power;
pub mod reward;
pub mod system;
pub mod verifreg;

use cid::Cid;
pub use fil_actor_reward_state::v8::AwardBlockRewardParams;
pub use fil_actors_shared::v9::builtin::singletons::{BURNT_FUNDS_ACTOR_ADDR, CHAOS_ACTOR_ADDR};
use fvm_shared::address::Address;
pub use fvm_shared::{clock::EPOCH_DURATION_SECONDS, smooth::FilterEstimate};
pub use known_cids::KNOWN_CIDS;

pub const RESERVE_ADDRESS: Address = Address::new_id(90);

/// Returns true if the code belongs to an account actor.
pub fn is_account_actor(code: &Cid) -> bool {
    account::is_v8_account_cid(code)
        || account::is_v9_account_cid(code)
        || account::is_v10_account_cid(code)
        || account::is_v11_account_cid(code)
}

/// Returns true if the code belongs to a placeholder actor.
pub fn is_placeholder_actor(code: &Cid) -> bool {
    placeholder::is_v10_placeholder_cid(code)
        || placeholder::is_v11_placeholder_cid(code)
        || placeholder::is_v12_placeholder_cid(code)
}

/// Returns true if the code belongs to a ethereum account actor.
pub fn is_eth_account_actor(code: &Cid) -> bool {
    ethaccount::is_v10_ethaccount_cid(code)
        || ethaccount::is_v11_ethaccount_cid(code)
        || ethaccount::is_v12_ethaccount_cid(code)
}
