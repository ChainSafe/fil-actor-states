pub mod actors;
pub mod network;

use anyhow::{Result, bail};
use serde_json::Value;

/// Supported actor types for decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    DataCap,
    VerifiedRegistry,
}

/// Actor state versions, corresponding to network upgrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorVersion {
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
}

/// Decode CBOR-encoded params for a given actor method.
pub fn decode_params(
    actor: ActorType,
    version: ActorVersion,
    method_num: u64,
    raw_bytes: &[u8],
) -> Result<Value> {
    match actor {
        ActorType::DataCap => actors::datacap::decode_params(version, method_num, raw_bytes),
        ActorType::VerifiedRegistry => {
            actors::verifreg::decode_params(version, method_num, raw_bytes)
        }
    }
}

/// Decode CBOR-encoded return value for a given actor method.
pub fn decode_return(
    actor: ActorType,
    version: ActorVersion,
    method_num: u64,
    raw_bytes: &[u8],
) -> Result<Value> {
    match actor {
        ActorType::DataCap => actors::datacap::decode_return(version, method_num, raw_bytes),
        ActorType::VerifiedRegistry => {
            actors::verifreg::decode_return(version, method_num, raw_bytes)
        }
    }
}

impl std::str::FromStr for ActorType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "datacap" | "f07" | "7" => Ok(ActorType::DataCap),
            "verifreg" | "verifiedregistry" | "verified_registry" | "f06" | "6" => {
                Ok(ActorType::VerifiedRegistry)
            }
            _ => bail!("Unknown actor type: {s}. Supported: datacap (f07), verifreg (f06)"),
        }
    }
}

impl std::str::FromStr for ActorVersion {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "v9" | "9" => Ok(ActorVersion::V9),
            "v10" | "10" => Ok(ActorVersion::V10),
            "v11" | "11" => Ok(ActorVersion::V11),
            "v12" | "12" => Ok(ActorVersion::V12),
            "v13" | "13" => Ok(ActorVersion::V13),
            "v14" | "14" => Ok(ActorVersion::V14),
            "v15" | "15" => Ok(ActorVersion::V15),
            "v16" | "16" => Ok(ActorVersion::V16),
            "v17" | "17" => Ok(ActorVersion::V17),
            _ => bail!("Unknown actor version: {s}. Supported: v9..v17"),
        }
    }
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorType::DataCap => write!(f, "datacap"),
            ActorType::VerifiedRegistry => write!(f, "verifreg"),
        }
    }
}

impl std::fmt::Display for ActorVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorVersion::V9 => write!(f, "v9"),
            ActorVersion::V10 => write!(f, "v10"),
            ActorVersion::V11 => write!(f, "v11"),
            ActorVersion::V12 => write!(f, "v12"),
            ActorVersion::V13 => write!(f, "v13"),
            ActorVersion::V14 => write!(f, "v14"),
            ActorVersion::V15 => write!(f, "v15"),
            ActorVersion::V16 => write!(f, "v16"),
            ActorVersion::V17 => write!(f, "v17"),
        }
    }
}
