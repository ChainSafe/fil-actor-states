// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared3::address::Address;
use fvm_shared3::econ::TokenAmount;

#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct AwardBlockRewardParams {
    pub miner: Address,
    pub penalty: TokenAmount,
    pub gas_reward: TokenAmount,
    pub win_count: i64,
}

pub use fvm_shared3::reward::ThisEpochRewardReturn;
