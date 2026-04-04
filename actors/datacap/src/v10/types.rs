// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared3::address::Address;
use fvm_shared3::econ::TokenAmount;

#[cfg_attr(feature = "json", derive(fil_actor_json_derive::IntoJsonValue))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct MintParams {
    // Recipient of the newly minted tokens.
    pub to: Address,
    // Amount of tokens to mint.
    pub amount: TokenAmount,
    // Addresses to be granted effectively-infinite operator allowance for the recipient.
    pub operators: Vec<Address>,
}

#[cfg_attr(feature = "json", derive(fil_actor_json_derive::IntoJsonValue))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct DestroyParams {
    pub owner: Address,
    pub amount: TokenAmount,
}

#[cfg_attr(feature = "json", derive(fil_actor_json_derive::IntoJsonValue))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct GranularityReturn {
    pub granularity: u64,
}
