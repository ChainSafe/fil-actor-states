// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{self, AtomicBool},
};

use anyhow::Context;

/// Ensures go mod prepared for compiling / running go tests
pub fn ensure_go_mod_prepared() {
    static CHECKED: AtomicBool = AtomicBool::new(false);
    lazy_static::lazy_static! {
        static ref LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    }

    if !CHECKED.load(atomic::Ordering::Relaxed) {
        let _guard = LOCK.lock();
        if !CHECKED.load(atomic::Ordering::Relaxed) {
            let cwd = std::env::current_dir().unwrap();
            println!(
                "Setting up go mod... pid: {}, pwd: {}",
                std::process::id(),
                cwd.display()
            );
            const ERROR_CONTEXT: &str = "Fail to prepare `go` test dependencies, make sure you have `Go` compiler (version defined in `go_compat/go.mod`) installed and available in $PATH. For details refer to instructions at <https://go.dev/doc/install>";
            Command::new("go")
                .args(["mod", "vendor"])
                .current_dir(go_compat_dir().unwrap())
                .output()
                .context(ERROR_CONTEXT)
                .unwrap();
            CHECKED.store(true, atomic::Ordering::Relaxed);
        }
    }
}

/// Helper for locating `go_compat` dir
pub fn go_compat_dir() -> anyhow::Result<PathBuf> {
    const GO_COMPAT_DIR_NAME: &str = "go_compat";
    const GO_MOD_FILE_NAME: &str = "go.mod";

    let go_compat_dir = (|| {
        let mut cwd: PathBuf = std::env::current_dir()?;
        loop {
            let dir = cwd.join(GO_COMPAT_DIR_NAME);
            if dir.join(GO_MOD_FILE_NAME).is_file() {
                return Ok::<_, anyhow::Error>(dir);
            }
            cwd = cwd
                .parent()
                .context("Failed to locate go compat dir")?
                .to_path_buf();
        }
    })()?;

    Ok(go_compat_dir)
}

/// Helper for locating `go_compat/tests` dir
pub fn go_compat_tests_dir() -> anyhow::Result<PathBuf> {
    Ok(go_compat_dir()?.join("tests"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::*;

    #[test]
    fn test_locate_go_compat_dir() -> Result<()> {
        ensure!(go_compat_dir()?.is_dir());
        Ok(())
    }

    #[test]
    fn test_ensure_go_mod_prepared() -> Result<()> {
        ensure_go_mod_prepared();
        ensure!(go_compat_dir()?.join("vendor").is_dir());
        Ok(())
    }
}
