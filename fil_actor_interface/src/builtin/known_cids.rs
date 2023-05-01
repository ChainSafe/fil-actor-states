// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::r#mod::cid_serde;
use anyhow::{bail, Ok, Result};
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
pub struct CidPerVersion {
    #[serde(with = "cid_serde")]
    pub v8: Cid,
    #[serde(with = "cid_serde")]
    pub v9: Cid,
    #[serde(with = "cid_serde")]
    pub v10: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum CidsPerNetworkVersion {
    E1 {
        v8: CidPerNetwork,
        v9: CidPerNetwork,
        v10: CidPerNetwork,
        v11: CidPerNetwork,
    },
    E2 {
        v9: CidPerNetwork,
        v10: CidPerNetwork,
        v11: CidPerNetwork,
    },
    E3 {
        v10: CidPerNetwork,
        v11: CidPerNetwork,
    },
    E4 {
        v11: CidPerNetwork,
    },
}

impl CidsPerNetworkVersion {
    pub fn v8(&self) -> Result<&CidPerNetwork> {
        match self {
            CidsPerNetworkVersion::E1 { v8, .. } => Ok(v8),
            _ => bail!("V8 Not Supported for {:?}", self),
        }
    }

    pub fn v9(&self) -> Result<&CidPerNetwork> {
        match self {
            CidsPerNetworkVersion::E1 { v9, .. } => Ok(v9),
            CidsPerNetworkVersion::E2 { v9, .. } => Ok(v9),
            _ => bail!("V9 Not Supported for {:?}", self),
        }
    }

    pub fn v10(&self) -> Result<&CidPerNetwork> {
        match self {
            CidsPerNetworkVersion::E1 { v10, .. } => Ok(v10),
            CidsPerNetworkVersion::E2 { v10, .. } => Ok(v10),
            CidsPerNetworkVersion::E3 { v10, .. } => Ok(v10),
            _ => bail!("V10 Not Supported for {:?}", self),
        }
    }

    pub fn v11(&self) -> Result<&CidPerNetwork> {
        match self {
            CidsPerNetworkVersion::E1 { v11, .. } => Ok(v11),
            CidsPerNetworkVersion::E2 { v11, .. } => Ok(v11),
            CidsPerNetworkVersion::E3 { v11, .. } => Ok(v11),
            CidsPerNetworkVersion::E4 { v11 } => Ok(v11),
        }
    }
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
        ensure!(crate::KNOWN_CIDS.actor.market.v8()?.contains(
            &Cid::try_from("bafk2bzacediohrxkp2fbsl4yj4jlupjdkgsiwqb4zuezvinhdo2j5hrxco62q")
                .unwrap()
        ));
        ensure!(!crate::KNOWN_CIDS.actor.market.v9()?.contains(
            &Cid::try_from("bafk2bzacediohrxkp2fbsl4yj4jlupjdkgsiwqb4zuezvinhdo2j5hrxco62q")
                .unwrap()
        ));
        ensure!(crate::KNOWN_CIDS.actor.placeholder.v8().is_err());
        ensure!(
            crate::KNOWN_CIDS.actor.market.v8()?.calibnet
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
