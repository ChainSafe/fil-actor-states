// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use serde::{Deserialize, Serialize};

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

mod cid_serde {
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
