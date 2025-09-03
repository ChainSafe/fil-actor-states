// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Enum representing supported actor versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActorVersion {
    /// Version 8
    V8,
    /// Version 9
    V9,
    /// Version 10
    V10,
    /// Version 11
    V11,
    /// Version 12
    V12,
    /// Version 13
    V13,
    /// Version 14
    V14,
    /// Version 15
    V15,
    /// Version 16
    V16,
}

impl ActorVersion {
    pub fn from_u64(version: u64) -> Result<Self> {
        match version {
            8 => Ok(ActorVersion::V8),
            9 => Ok(ActorVersion::V9),
            10 => Ok(ActorVersion::V10),
            11 => Ok(ActorVersion::V11),
            12 => Ok(ActorVersion::V12),
            13 => Ok(ActorVersion::V13),
            14 => Ok(ActorVersion::V14),
            15 => Ok(ActorVersion::V15),
            16 => Ok(ActorVersion::V16),
            _ => Err(anyhow!("Unsupported actor version: {}", version)),
        }
    }

    pub fn to_u64(self) -> u64 {
        match self {
            ActorVersion::V8 => 8,
            ActorVersion::V9 => 9,
            ActorVersion::V10 => 10,
            ActorVersion::V11 => 11,
            ActorVersion::V12 => 12,
            ActorVersion::V13 => 13,
            ActorVersion::V14 => 14,
            ActorVersion::V15 => 15,
            ActorVersion::V16 => 16,
        }
    }
}

impl Display for ActorVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.to_u64())
    }
}

impl TryFrom<u64> for ActorVersion {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self> {
        Self::from_u64(value)
    }
}

impl From<ActorVersion> for u64 {
    fn from(version: ActorVersion) -> Self {
        version.to_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_version_conversion() {
        // Test all supported versions
        let test_cases = [
            (8, ActorVersion::V8),
            (9, ActorVersion::V9),
            (10, ActorVersion::V10),
            (11, ActorVersion::V11),
            (12, ActorVersion::V12),
            (13, ActorVersion::V13),
            (14, ActorVersion::V14),
            (15, ActorVersion::V15),
            (16, ActorVersion::V16),
        ];

        for (number, expected_version) in test_cases.iter() {
            // Test from_u64
            let version = ActorVersion::from_u64(*number).unwrap();
            assert_eq!(version, *expected_version);

            // Test to_u64
            assert_eq!(version.to_u64(), *number);

            // Test TryFrom/Into traits
            let version_try_from: ActorVersion = (*number).try_into().unwrap();
            assert_eq!(version_try_from, *expected_version);

            let number_into: u64 = (*expected_version).into();
            assert_eq!(number_into, *number);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(ActorVersion::V8.to_string(), "v8");
        assert_eq!(ActorVersion::V16.to_string(), "v16");
    }
}
