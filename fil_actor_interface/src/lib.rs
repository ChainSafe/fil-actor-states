// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

mod builtin;
mod convert;
mod io;
mod r#mod;

pub use self::builtin::*;
pub use r#mod::NetworkManifest;

const NETWORK_MANIFESTS_JSON_STR: &str =
    include_str!(concat!(env!("OUT_DIR"), "/network_manifests.json"));

lazy_static::lazy_static! {
    pub static ref NETWORK_MANIFESTS: Vec<NetworkManifest> = serde_json::from_str(NETWORK_MANIFESTS_JSON_STR).expect("Fail to parse NETWORK_MANIFESTS");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_manifests() {
        assert!(!NETWORK_MANIFESTS.is_empty())
    }
}
