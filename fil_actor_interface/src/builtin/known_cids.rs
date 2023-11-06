// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::r#mod::cid_serde;
use cid::Cid;
use multihash::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};

const RAW: u64 = 0x55;

lazy_static::lazy_static! {
    static ref MANIFEST_CIDS: ManifestCids = serde_yaml::from_str(include_str!("manifest_cids.yaml")).unwrap();
    static ref ACTOR_CIDS: ActorCids = serde_yaml::from_str(include_str!("actor_cids.yaml")).unwrap();
    pub static ref KNOWN_CIDS: KnownCids = KnownCids {
        manifest: MANIFEST_CIDS.clone(),
        actor: ACTOR_CIDS.clone()
    };
    pub static ref INIT_V0_ACTOR_CID: Cid = make_builtin(b"fil/1/init");
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
    pub devnet: CidPerVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ActorCids {
    pub account: V8Onwards,
    pub cron: V8Onwards,
    pub market: V8Onwards,
    pub datacap: V9Onwards,
    pub ethaccount: V10Onwards,
    pub evm: V10Onwards,
    pub init: V8Onwards,
    pub miner: V8Onwards,
    pub multisig: V8Onwards,
    pub placeholder: V10Onwards,
    pub power: V8Onwards,
    pub reward: V8Onwards,
    pub system: V8Onwards,
    pub verifreg: V8Onwards,
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
    #[serde(with = "cid_serde")]
    pub v12: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V8Onwards {
    pub v8: CidPerNetwork,
    pub v9: CidPerNetwork,
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
    pub v12: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V9Onwards {
    pub v9: CidPerNetwork,
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
    pub v12: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V10Onwards {
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
    pub v12: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CidPerNetwork {
    #[serde(default, with = "cid_serde")]
    pub mainnet: Cid,
    #[serde(default, with = "cid_serde")]
    pub calibnet: Cid,
    #[serde(default, with = "cid_serde")]
    pub devnet: Cid,
}

impl CidPerNetwork {
    pub fn contains(&self, cid: &Cid) -> bool {
        self.mainnet == *cid || self.calibnet == *cid || self.devnet == *cid
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{ensure, Ok, Result};

    use super::*;

    #[test]
    fn test_loading_static_value() -> Result<()> {
        let serialized = serde_yaml::to_string(&*KNOWN_CIDS)?;
        let deserialized = serde_yaml::from_str(&serialized)?;
        ensure!(&*KNOWN_CIDS == &deserialized);

        Ok(())
    }
}

fn make_builtin(bz: &[u8]) -> Cid {
    Cid::new_v1(RAW, Code::Identity.digest(bz))
}
