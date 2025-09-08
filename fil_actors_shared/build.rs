// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::WalkDir;

fn main() {
    if env::var("SKIP_ACTOR_VERSION_CHECK").is_ok() {
        println!("cargo:warning=Skipping actor version validation during publish.");
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let actors_dir = manifest_dir.join("../actors");
    let versions_file = manifest_dir.join("src/actor_versions.rs");

    println!("cargo:rerun-if-changed={}", actors_dir.to_str().unwrap());
    println!("cargo:rerun-if-changed={}", versions_file.to_str().unwrap());

    verify_actor_versions(&actors_dir, &versions_file);
}

/// extract version numbers from the enum.
fn get_enum_versions(path: &Path) -> HashSet<u8> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("❌ BUILD FAILED: Could not read {:?}: {}", path, e));

    // Regex to find lines like `V12,` or `V13 = 13,`
    let re = Regex::new(r"V(\d+)").unwrap();

    re.captures_iter(&content)
        .map(|cap| cap[1].parse().expect("Failed to parse version number"))
        .collect()
}

/// Scans the `actors` directory to find all unique version subdirectories (e.g., "v8").
/// Excludes v0 as it's a legacy version not included in the main ActorVersion enum.
fn get_directory_versions(base_path: &Path) -> HashSet<u8> {
    WalkDir::new(base_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
        .filter_map(|entry| {
            let dir_name = entry.file_name().to_string_lossy();
            if let Some(version_str) = dir_name.strip_prefix('v') {
                if let Ok(v_num) = version_str.parse::<u8>() {
                    if v_num > 0 {
                        return Some(v_num);
                    }
                }
            }
            None
        })
        .collect()
}

fn verify_actor_versions(actors_dir: &Path, versions_file: &Path) {
    if !actors_dir.exists() {
        panic!(
            "❌ BUILD FAILED: Actors directory not found at {:?}",
            actors_dir
        );
    }
    if !versions_file.exists() {
        panic!(
            "❌ BUILD FAILED: Actor versions file not found at {:?}",
            versions_file
        );
    }

    let versions_from_enum = get_enum_versions(versions_file);
    let versions_from_dirs = get_directory_versions(actors_dir);

    let missing_in_enum: Vec<_> = versions_from_dirs.difference(&versions_from_enum).collect();

    let missing_in_dirs: Vec<_> = versions_from_enum.difference(&versions_from_dirs).collect();

    if !missing_in_enum.is_empty() {
        panic!(
            "❌ BUILD FAILED: Found version directories that are not in ActorVersion enum: {:?}.\n\
             Please add these versions to `fil_actors_shared/src/actor_versions.rs`.",
            missing_in_enum
        );
    }

    if !missing_in_dirs.is_empty() {
        panic!(
            "❌ BUILD FAILED: Found versions in ActorVersion enum that don't have corresponding directories: {:?}.\n\
             Please either remove these versions from the enum or add the missing actor version directories.",
            missing_in_dirs
        );
    }

    println!(
        "✅ Actor versions check passed! Found versions: {:?}",
        versions_from_enum
    );
}
