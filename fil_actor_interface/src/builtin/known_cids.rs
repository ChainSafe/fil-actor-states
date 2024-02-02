// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{cids_filename, r#mod::cid_serde, NetworkManifest};
use cid::Cid;
use multihash::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};

const RAW: u64 = 0x55;

lazy_static::lazy_static! {
    static ref MANIFEST_CIDS: ManifestCids = serde_yaml::from_str(include_str!(concat!(env!("OUT_DIR"), "/", cids_filename!(manifests)))).unwrap();
    static ref ACTOR_CIDS: ActorCids = serde_yaml::from_str(include_str!(concat!(env!("OUT_DIR"), "/", cids_filename!(actors)))).unwrap();
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

fn find_network_manifest(manifests: &[NetworkManifest], network: &str, version: i64) -> Cid {
    manifests
        .iter()
        .find(|m| m.network == network && m.version == version)
        .map(|m| m.manifest_cid)
        .unwrap()
}

fn create_manifest_cids_per_version(manifests: &[NetworkManifest], network: &str) -> CidPerVersion {
    CidPerVersion {
        v8: find_network_manifest(manifests, network, 8),
        v9: find_network_manifest(manifests, network, 9),
        v10: find_network_manifest(manifests, network, 10),
        v11: find_network_manifest(manifests, network, 11),
        v12: find_network_manifest(manifests, network, 12),
        v13: find_network_manifest(manifests, network, 13),
    }
}

macro_rules! create_actor_cid_for_network {
    ($manifests:ident, $version:literal,$actor:tt) => {
        $manifests
            .iter()
            .find(|m| m.network == "mainnet" && m.version == 8)
            .unwrap()
            .actors
            .$actor
    };
}

macro_rules! create_cids_per_network {
    ($manifests:ident, $actor:tt, $version:literal) => {
        CidPerNetwork {
            mainnet: create_actor_cid_for_network!($manifests, $version, $actor),
            calibnet: create_actor_cid_for_network!($manifests, $version, $actor),
            devnet: create_actor_cid_for_network!($manifests, $version, $actor),
            butterflynet: create_actor_cid_for_network!($manifests, $version, $actor),
        }
    };
}

macro_rules! create_cids_per_network_option {
    ($manifests:ident, $actor:tt, $version:literal) => {
        CidPerNetwork {
            mainnet: create_actor_cid_for_network!($manifests, $version, $actor).unwrap(),
            calibnet: create_actor_cid_for_network!($manifests, $version, $actor).unwrap(),
            devnet: create_actor_cid_for_network!($manifests, $version, $actor).unwrap(),
            butterflynet: create_actor_cid_for_network!($manifests, $version, $actor).unwrap(),
        }
    };
}

macro_rules! create_actor_cids_per_version {
    ("V8Onwards", $manifests:ident,$actor:tt) => {
        V8Onwards {
            v8: create_cids_per_network!($manifests, $actor, 8),
            v9: create_cids_per_network!($manifests, $actor, 9),
            v10: create_cids_per_network!($manifests, $actor, 10),
            v11: create_cids_per_network!($manifests, $actor, 11),
            v12: create_cids_per_network!($manifests, $actor, 12),
            v13: create_cids_per_network!($manifests, $actor, 13),
        }
    };
    ("V9Onwards", $manifests:ident,$actor:tt) => {
        V9Onwards {
            v9: create_cids_per_network_option!($manifests, $actor, 9),
            v10: create_cids_per_network_option!($manifests, $actor, 10),
            v11: create_cids_per_network_option!($manifests, $actor, 11),
            v12: create_cids_per_network_option!($manifests, $actor, 12),
            v13: create_cids_per_network_option!($manifests, $actor, 13),
        }
    };
    ("V10Onwards", $manifests:ident,$actor:tt) => {
        V10Onwards {
            v10: create_cids_per_network_option!($manifests, $actor, 10),
            v11: create_cids_per_network_option!($manifests, $actor, 11),
            v12: create_cids_per_network_option!($manifests, $actor, 12),
            v13: create_cids_per_network_option!($manifests, $actor, 13),
        }
    };
}

impl From<Vec<NetworkManifest>> for KnownCids {
    fn from(manifests: Vec<NetworkManifest>) -> Self {
        let manifest_cids = ManifestCids {
            mainnet: create_manifest_cids_per_version(&manifests, "mainnet"),
            calibnet: create_manifest_cids_per_version(&manifests, "calibrationnet"),
            devnet: create_manifest_cids_per_version(&manifests, "devnet"),
            butterflynet: create_manifest_cids_per_version(&manifests, "butterflynet"),
        };

        let actor_cids = ActorCids {
            account: create_actor_cids_per_version!("V8Onwards", manifests, account),
            cron: create_actor_cids_per_version!("V8Onwards", manifests, cron),
            market: create_actor_cids_per_version!("V8Onwards", manifests, storagemarket),
            datacap: create_actor_cids_per_version!("V9Onwards", manifests, datacap),
            ethaccount: create_actor_cids_per_version!("V10Onwards", manifests, ethaccount),
            evm: create_actor_cids_per_version!("V10Onwards", manifests, evm),
            init: create_actor_cids_per_version!("V8Onwards", manifests, init),
            miner: create_actor_cids_per_version!("V8Onwards", manifests, storageminer),
            multisig: create_actor_cids_per_version!("V8Onwards", manifests, multisig),
            placeholder: create_actor_cids_per_version!("V10Onwards", manifests, placeholder),
            power: create_actor_cids_per_version!("V8Onwards", manifests, storagepower),
            reward: create_actor_cids_per_version!("V8Onwards", manifests, reward),
            system: create_actor_cids_per_version!("V8Onwards", manifests, system),
            verifreg: create_actor_cids_per_version!("V8Onwards", manifests, verifiedregistry),
            paymentchannel: create_actor_cids_per_version!("V8Onwards", manifests, paymentchannel),
        };

        KnownCids {
            manifest: manifest_cids,
            actor: actor_cids,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ManifestCids {
    pub mainnet: CidPerVersion,
    pub calibnet: CidPerVersion,
    pub devnet: CidPerVersion,
    pub butterflynet: CidPerVersion,
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
    pub paymentchannel: V8Onwards,
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
    #[serde(with = "cid_serde")]
    pub v13: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V8Onwards {
    pub v8: CidPerNetwork,
    pub v9: CidPerNetwork,
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
    pub v12: CidPerNetwork,
    pub v13: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V9Onwards {
    pub v9: CidPerNetwork,
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
    pub v12: CidPerNetwork,
    pub v13: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct V10Onwards {
    pub v10: CidPerNetwork,
    pub v11: CidPerNetwork,
    pub v12: CidPerNetwork,
    pub v13: CidPerNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CidPerNetwork {
    #[serde(default, with = "cid_serde")]
    pub mainnet: Cid,
    #[serde(default, with = "cid_serde")]
    pub calibnet: Cid,
    #[serde(default, with = "cid_serde")]
    pub devnet: Cid,
    #[serde(default, with = "cid_serde")]
    pub butterflynet: Cid,
}

impl CidPerNetwork {
    pub fn contains(&self, cid: &Cid) -> bool {
        self.mainnet == *cid
            || self.calibnet == *cid
            || self.devnet == *cid
            || self.butterflynet == *cid
    }
}

fn make_builtin(bz: &[u8]) -> Cid {
    Cid::new_v1(RAW, Code::Identity.digest(bz))
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
