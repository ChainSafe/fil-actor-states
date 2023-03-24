// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

//! This build script generates test cases from https://github.com/filecoin-project/lotus/blob/master/build/builtin_actors_gen.go

mod src;
use src::NetworkManifest;

use std::path::Path;

use regex::Regex;

const GO_SRC: &str = include_str!("builtin_actors_gen.go");

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    let json = to_json()?;

    std::fs::write(
        out_dir.join("network_manifests.json"),
        serde_json::to_string_pretty(&json)?,
    )?;

    let mut combinations = vec![];
    for cfg in json {
        match cfg.network.as_str() {
            "mainnet" | "calibrationnet" => {}
            _ => {
                continue;
            }
        }

        fn normalize(network: &str) -> String {
            network.replace('-', "_")
        }

        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "account",
            cfg.actors.account,
        ));
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "cron",
            cfg.actors.cron,
        ));
        if let Some(cid) = cfg.actors.datacap {
            if cfg.version == 10 {
                combinations.push((normalize(&cfg.network), cfg.version, "datacap", cid));
            }
        }
        if let Some(cid) = cfg.actors.ethaccount {
            combinations.push((normalize(&cfg.network), cfg.version, "ethaccount", cid));
        }
        if let Some(cid) = cfg.actors.evm {
            combinations.push((normalize(&cfg.network), cfg.version, "evm", cid));
        }
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "init",
            cfg.actors.init,
        ));
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "market",
            cfg.actors.storagemarket,
        ));
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "miner",
            cfg.actors.storageminer,
        ));
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "multisig",
            cfg.actors.multisig,
        ));
        if let Some(cid) = cfg.actors.placeholder {
            combinations.push((normalize(&cfg.network), cfg.version, "placeholder", cid));
        }
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "power",
            cfg.actors.storagepower,
        ));
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "reward",
            cfg.actors.reward,
        ));
        combinations.push((
            normalize(&cfg.network),
            cfg.version,
            "system",
            cfg.actors.system,
        ));
    }

    let mut tests = String::new();
    for (network, version, actor, cid) in combinations {
        tests.push_str(&format!(
            "
        
        #[test]
        fn test_{actor}_{}_{version} () {{
            assert!(is_v{version}_{actor}_cid(&cid::Cid::try_from(\"{cid}\").unwrap()));
        }}

        ",
            network.replace('-', "_"),
        ));
    }

    std::fs::write(out_dir.join("network_manifest_tests.rs"), tests)?;
    Ok(())
}

fn to_json() -> anyhow::Result<Vec<NetworkManifest>> {
    // Extract json-like str from go map
    let json = {
        let pattern = Regex::new(r"BuiltinActorsMetadata\{(?P<json>.+)\}")?;
        let normalized_go_src = GO_SRC.replace('\n', "");
        let captures = pattern.captures(&normalized_go_src).unwrap();
        captures.name("json").unwrap().as_str().to_owned()
    };
    // Wrap into json array
    let json = format!("[{json}]");
    // Remove map declarations
    let json = {
        let pattern = Regex::new(r"map\[string\]cid\.Cid")?;
        pattern.replace_all(&json, "").to_string()
    };
    // Remove function calls around CIDs
    let json = {
        let pattern = Regex::new(r"MustParseCid\((.+?)\)")?;
        pattern.replace_all(&json, "$1").to_string()
    };
    // Wrap keys with double quotes
    let json = {
        let pattern = Regex::new(r"(\w+):")?;
        pattern.replace_all(&json, "\"$1\":").to_string()
    };
    // Remove trailing commas
    let json = {
        let pattern = Regex::new(r",\s*}")?;
        pattern.replace_all(&json, "}").to_string()
    };
    // Parse json
    let json: Vec<NetworkManifest> = serde_json::from_str(&json)?;

    Ok(json)
}
