// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::{Deserialize, Serialize};

/// Enum representing supported actor versions.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::FromRepr,
)]
#[repr(u8)]
#[strum(serialize_all = "lowercase")]
pub enum ActorVersion {
    V8 = 8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
}
