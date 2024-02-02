// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! cids_filename {
    (actors) => {
        "actor_cids.yaml"
    };
    (manifests) => {
        "manifest_cids.yaml"
    };
}

#[derive(Serialize, Deserialize)]
pub struct NetworkManifest {
    #[serde(rename = "Network")]
    pub network: String,
    #[serde(rename = "Version")]
    pub version: i64,
    #[serde(rename = "ManifestCid", with = "cid_serde")]
    pub manifest_cid: Cid,
    #[serde(rename = "Actors")]
    pub actors: NetworkManifestActors,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkManifestActors {
    #[serde(with = "cid_serde")]
    pub account: Cid,
    #[serde(with = "cid_serde")]
    pub cron: Cid,
    #[serde(with = "cid_serde_opt", default)]
    pub datacap: Option<Cid>,
    #[serde(with = "cid_serde_opt", default)]
    pub eam: Option<Cid>,
    #[serde(with = "cid_serde_opt", default)]
    pub ethaccount: Option<Cid>,
    #[serde(with = "cid_serde_opt", default)]
    pub evm: Option<Cid>,
    #[serde(with = "cid_serde")]
    pub init: Cid,
    #[serde(with = "cid_serde")]
    pub multisig: Cid,
    #[serde(with = "cid_serde")]
    pub paymentchannel: Cid,
    #[serde(with = "cid_serde_opt", default)]
    pub placeholder: Option<Cid>,
    #[serde(with = "cid_serde")]
    pub reward: Cid,
    #[serde(with = "cid_serde")]
    pub storagemarket: Cid,
    #[serde(with = "cid_serde")]
    pub storageminer: Cid,
    #[serde(with = "cid_serde")]
    pub storagepower: Cid,
    #[serde(with = "cid_serde")]
    pub system: Cid,
    #[serde(with = "cid_serde")]
    pub verifiedregistry: Cid,
}

pub mod cid_serde {
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(value: &Cid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Cid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cid = String::deserialize(deserializer)?;
        Cid::try_from(cid).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

mod cid_serde_opt {
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(value: &Option<Cid>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(value) = value {
            value.to_string().serialize(serializer)
        } else {
            ().serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Cid>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(cid) = String::deserialize(deserializer) {
            Ok(Some(
                Cid::try_from(cid).map_err(|e| serde::de::Error::custom(e.to_string()))?,
            ))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct KnownCids {
    pub manifest: ManifestCids,
    pub actor: ActorCids,
}

/// Finds the manifest CID for a given network and version. Panics if not found.
fn find_network_manifest(manifests: &[NetworkManifest], network: &str, version: i64) -> Cid {
    manifests
        .iter()
        .find(|m| m.network == network && m.version == version)
        .map(|m| m.manifest_cid)
        .expect("manifest not found")
}

/// Creates a all manifest CIDs for a given network (by versions).
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

/// Grabs the actor CID for a given network and version. Panics if not found.
macro_rules! create_actor_cid_for_network {
    ($manifests:ident, $network:literal, $version:literal, $actor:tt) => {
        $manifests
            .iter()
            .find(|m| m.network == $network && m.version == $version)
            .expect("manifest not found")
            .actors
            .$actor
    };
}

/// Grabs all actor CIDs for a given actor and version (by network). Only works for actors that
/// exist in all networks (i.e, are not optional).
macro_rules! create_actor_cids_per_network {
    ($manifests:ident, $actor:tt, $version:literal) => {
        CidPerNetwork {
            mainnet: create_actor_cid_for_network!($manifests, "mainnet", $version, $actor),
            calibnet: create_actor_cid_for_network!($manifests, "calibrationnet", $version, $actor),
            devnet: create_actor_cid_for_network!($manifests, "devnet", $version, $actor),
            butterflynet: create_actor_cid_for_network!(
                $manifests,
                "butterflynet",
                $version,
                $actor
            ),
        }
    };
}

/// Grabs all actor CIDs for a given actor and version (by network). Works for optional actors,
/// i.e., actors that are not present in all networks (like EAM).
macro_rules! create_actor_cids_per_network_optional {
    ($manifests:ident, $actor:tt, $version:literal) => {
        CidPerNetwork {
            mainnet: create_actor_cid_for_network!($manifests, "mainnet", $version, $actor)
                .expect("actor not found"),
            calibnet: create_actor_cid_for_network!($manifests, "calibrationnet", $version, $actor)
                .expect("actor not found"),
            devnet: create_actor_cid_for_network!($manifests, "devnet", $version, $actor)
                .expect("actor not found"),
            butterflynet: create_actor_cid_for_network!(
                $manifests,
                "butterflynet",
                $version,
                $actor
            )
            .expect("actor not found"),
        }
    };
}

/// Grabs all actor CIDs for a given actor, by version.
macro_rules! create_actor_cids_per_version {
    (V8Onwards, $manifests:ident,$actor:tt) => {
        V8Onwards {
            v8: create_actor_cids_per_network!($manifests, $actor, 8),
            v9: create_actor_cids_per_network!($manifests, $actor, 9),
            v10: create_actor_cids_per_network!($manifests, $actor, 10),
            v11: create_actor_cids_per_network!($manifests, $actor, 11),
            v12: create_actor_cids_per_network!($manifests, $actor, 12),
            v13: create_actor_cids_per_network!($manifests, $actor, 13),
        }
    };
    (V9Onwards, $manifests:ident,$actor:tt) => {
        V9Onwards {
            v9: create_actor_cids_per_network_optional!($manifests, $actor, 9),
            v10: create_actor_cids_per_network_optional!($manifests, $actor, 10),
            v11: create_actor_cids_per_network_optional!($manifests, $actor, 11),
            v12: create_actor_cids_per_network_optional!($manifests, $actor, 12),
            v13: create_actor_cids_per_network_optional!($manifests, $actor, 13),
        }
    };
    (V10Onwards, $manifests:ident,$actor:tt) => {
        V10Onwards {
            v10: create_actor_cids_per_network_optional!($manifests, $actor, 10),
            v11: create_actor_cids_per_network_optional!($manifests, $actor, 11),
            v12: create_actor_cids_per_network_optional!($manifests, $actor, 12),
            v13: create_actor_cids_per_network_optional!($manifests, $actor, 13),
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
            account: create_actor_cids_per_version!(V8Onwards, manifests, account),
            cron: create_actor_cids_per_version!(V8Onwards, manifests, cron),
            market: create_actor_cids_per_version!(V8Onwards, manifests, storagemarket),
            datacap: create_actor_cids_per_version!(V9Onwards, manifests, datacap),
            ethaccount: create_actor_cids_per_version!(V10Onwards, manifests, ethaccount),
            evm: create_actor_cids_per_version!(V10Onwards, manifests, evm),
            init: create_actor_cids_per_version!(V8Onwards, manifests, init),
            miner: create_actor_cids_per_version!(V8Onwards, manifests, storageminer),
            multisig: create_actor_cids_per_version!(V8Onwards, manifests, multisig),
            placeholder: create_actor_cids_per_version!(V10Onwards, manifests, placeholder),
            power: create_actor_cids_per_version!(V8Onwards, manifests, storagepower),
            reward: create_actor_cids_per_version!(V8Onwards, manifests, reward),
            system: create_actor_cids_per_version!(V8Onwards, manifests, system),
            verifreg: create_actor_cids_per_version!(V8Onwards, manifests, verifiedregistry),
            paymentchannel: create_actor_cids_per_version!(V8Onwards, manifests, paymentchannel),
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
