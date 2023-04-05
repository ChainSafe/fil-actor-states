// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::r#mod::cid_serde;
use ahash::HashSet;
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
    #[serde(with = "cid_hashset")]
    pub v8: HashSet<Cid>,
    #[serde(with = "cid_hashset")]
    pub v9: HashSet<Cid>,
    #[serde(with = "cid_hashset")]
    pub v10: HashSet<Cid>,
}

mod cid_hashset {
    use ahash::HashSetExt;
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(value: &HashSet<Cid>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let transcoded = HashSet::from_iter(value.iter().map(|cid| cid.to_string()));
        transcoded.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashSet<Cid>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let transcoded: HashSet<String> = HashSet::deserialize(deserializer)?;
        let mut result = HashSet::with_capacity(transcoded.len());
        for cid in transcoded {
            result.insert(Cid::try_from(cid).map_err(|e| serde::de::Error::custom(e.to_string()))?);
        }
        Ok(result)
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
