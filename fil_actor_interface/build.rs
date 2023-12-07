// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

//! This build script generates test cases from https://github.com/filecoin-project/lotus/blob/master/build/builtin_actors_gen.go

mod src;
use src::{KnownCids, NetworkManifest};

use std::path::Path;

use regex::Regex;

const GO_SRC: &str = include_str!("builtin_actors_gen.go");

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    let json = to_json()?;

    let actors: KnownCids = json.into();
    let actors_yaml = serde_yaml::to_string(&actors.actor)?;
    std::fs::write(out_dir.join(cids_filename!(actors)), actors_yaml)?;

    let manifests_yaml = serde_yaml::to_string(&actors.manifest)?;
    std::fs::write(out_dir.join(cids_filename!(manifests)), manifests_yaml)?;

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
