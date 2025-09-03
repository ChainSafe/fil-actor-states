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
pub enum ActorVersion {
    #[strum(to_string = "v8")]
    V8 = 8,

    #[strum(to_string = "v9")]
    V9 = 9,

    #[strum(to_string = "v10")]
    V10 = 10,

    #[strum(to_string = "v11")]
    V11 = 11,

    #[strum(to_string = "v12")]
    V12 = 12,

    #[strum(to_string = "v13")]
    V13 = 13,

    #[strum(to_string = "v14")]
    V14 = 14,

    #[strum(to_string = "v15")]
    V15 = 15,

    #[strum(to_string = "v16")]
    V16 = 16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        assert_eq!(ActorVersion::V8.to_string(), "v8");
        assert_eq!(ActorVersion::V16.to_string(), "v16");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(ActorVersion::from_str("v8").unwrap(), ActorVersion::V8);
        assert_eq!(ActorVersion::from_str("v16").unwrap(), ActorVersion::V16);
        assert!(ActorVersion::from_str("v99").is_err());
    }

    #[test]
    fn test_from_repr() {
        assert_eq!(ActorVersion::from_repr(8), Some(ActorVersion::V8));
        assert_eq!(ActorVersion::from_repr(16), Some(ActorVersion::V16));
        assert_eq!(ActorVersion::from_repr(99), None);
    }

    #[test]
    fn test_basic_functionality() {
        assert!(ActorVersion::V8 < ActorVersion::V16);

        let version = ActorVersion::V10;
        match version {
            ActorVersion::V8 => panic!("Should not match V8"),
            ActorVersion::V10 => {} // This should match
            _ => panic!("Should match V10"),
        }
    }
}
