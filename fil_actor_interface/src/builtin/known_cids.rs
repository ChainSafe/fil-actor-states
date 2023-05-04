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
    pub mainnet: CidPerVersion,
    pub calibnet: CidPerVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ActorCids {
    pub account: V8Onwards,
    pub cron: V8Onwards,
    pub market: V8Onwards,
    pub datacap: V10Onwards,
    pub ethaccount: V10Onwards,
    pub evm: V10Onwards,
    pub init: V8Onwards,
    pub miner: V8Onwards,
    pub multisig: V8Onwards,
    pub placeholder: V10Onwards,
    pub power: V8Onwards,
    pub reward: V8Onwards,
    pub system: V8Onwards,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CidPerVersion {
    #[serde(with = "cid_serde")]
    pub v8: Cid,
    #[serde(with = "cid_serde")]
    pub v9: Cid,
    #[serde(with = "cid_serde")]
    pub v10: Cid,
    #[serde(with = "cid_serde")]
    pub v11: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V8Onwards {
    pub v8: CidPerNetwork,
    pub v9: CidPerNetwork,
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V10Onwards {
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
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
    use anyhow::{ensure, Ok, Result};

    use super::*;

    #[test]
    fn test_loading_static_value() -> Result<()> {
        ensure!(crate::KNOWN_CIDS.actor.market.v8.contains(
            &Cid::try_from("bafk2bzacediohrxkp2fbsl4yj4jlupjdkgsiwqb4zuezvinhdo2j5hrxco62q")
                .unwrap()
        ));
        ensure!(!crate::KNOWN_CIDS.actor.market.v9.contains(
            &Cid::try_from("bafk2bzacediohrxkp2fbsl4yj4jlupjdkgsiwqb4zuezvinhdo2j5hrxco62q")
                .unwrap()
        ));
        ensure!(
            crate::KNOWN_CIDS.actor.market.v8.calibnet
                == Cid::try_from("bafk2bzacebotg5coqnglzsdrqxtkqk2eq4krxt6zvds3i3vb2yejgxhexl2n6")
                    .unwrap()
        );

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
