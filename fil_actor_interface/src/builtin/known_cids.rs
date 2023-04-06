// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::r#mod::cid_serde;
use cid::Cid;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref MANIFEST_CIDS: ManifestCids = serde_yaml::from_str(include_str!("manifest_cids.yaml")).unwrap();
    static ref ACTOR_CIDS: ActorCids = serde_yaml::from_str(include_str!("actor_cids.yaml")).unwrap();
    pub static ref KNOWN_CIDS: KnownCids = KnownCids { manifest: MANIFEST_CIDS.clone(), actor: ACTOR_CIDS.clone() };
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct KnownCids {
    pub manifest: ManifestCids,
    pub actor: ActorCids,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ManifestCids {
    pub mainnet: CidPerNetworkVersion,
    pub calibnet: CidPerNetworkVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ActorCids {
    pub account: CidsPerNetworkVersion,
    pub cron: CidsPerNetworkVersion,
    pub market: CidsPerNetworkVersion,
    pub datacap: CidsPerNetworkVersion,
    pub ethaccount: CidsPerNetworkVersion,
    pub evm: CidsPerNetworkVersion,
    pub init: CidsPerNetworkVersion,
    pub miner: CidsPerNetworkVersion,
    pub multisig: CidsPerNetworkVersion,
    pub placeholder: CidsPerNetworkVersion,
    pub power: CidsPerNetworkVersion,
    pub reward: CidsPerNetworkVersion,
    pub system: CidsPerNetworkVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CidPerNetworkVersion {
    #[serde(with = "cid_serde")]
    pub v8: Cid,
    #[serde(with = "cid_serde")]
    pub v9: Cid,
    #[serde(with = "cid_serde")]
    pub v10: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CidsPerNetworkVersion {
    pub v8: CidPerNetwork,
    pub v9: CidPerNetwork,
    pub v10: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CidPerNetwork {
    #[serde(default, with = "cid_serde")]
    pub mainnet: Cid,
    #[serde(default, with = "cid_serde")]
    pub calibnet: Cid,
}

impl CidPerNetwork {
    pub fn contains(&self, cid: &Cid) -> bool {
        self.mainnet == *cid || self.calibnet == *cid
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{ensure, Result};

    use super::*;

    #[test]
    fn test_loading_static_value() -> Result<()> {
        ensure!(KNOWN_CIDS.actor.market.v8.contains(&Cid::try_from(
            "bafk2bzacediohrxkp2fbsl4yj4jlupjdkgsiwqb4zuezvinhdo2j5hrxco62q"
        )?));
        ensure!(!KNOWN_CIDS.actor.market.v9.contains(&Cid::try_from(
            "bafk2bzacediohrxkp2fbsl4yj4jlupjdkgsiwqb4zuezvinhdo2j5hrxco62q"
        )?));
        ensure!(
            KNOWN_CIDS.manifest.calibnet.v10
                == Cid::try_from("bafy2bzaced25ta3j6ygs34roprilbtb3f6mxifyfnm7z7ndquaruxzdq3y7lo")?
        );

        let serialized = serde_yaml::to_string(&*KNOWN_CIDS)?;
        let deserialized = serde_yaml::from_str(&serialized)?;
        ensure!(&*KNOWN_CIDS == &deserialized);

        Ok(())
    }
}
