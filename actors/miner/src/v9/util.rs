// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_hamt::BytesKey;
use fvm_shared3::sector::SectorNumber;

/// Converts sector number to hamt key
/// Go version: <https://github.com/filecoin-project/go-state-types/blob/master/builtin/v9/miner/miner_state.go#L319>
///
/// This function is needed for nv17 state migration, see <https://github.com/filecoin-project/go-state-types/blob/master/builtin/v9/migration/miner.go#L225>
pub fn sector_key(sector: SectorNumber) -> anyhow::Result<BytesKey> {
    let mut buffer = unsigned_varint::encode::u64_buffer();
    Ok(unsigned_varint::encode::u64(sector, &mut buffer)
        .to_vec()
        .into())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use fil_actors_test_utils::go_compat::go_compat_tests_dir;
    use pretty_assertions::assert_eq;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_sector_key(sector: SectorNumber) -> anyhow::Result<()> {
        let key = sector_key(sector)?;
        let key_hex = hex::encode(key.0);

        let app = Command::new("go")
            .args([
                "run",
                "actors/miner/v9/test_sector_key.go",
                "--sector",
                sector.to_string().as_str(),
            ])
            .current_dir(go_compat_tests_dir()?)
            .output()?;
        println!("Debug: {:#?}", app);
        if !app.stderr.is_empty() {
            println!("{}", String::from_utf8_lossy(&app.stderr));
            anyhow::bail!("Fail to run go test");
        }

        let key_hex_from_go = String::from_utf8_lossy(&app.stdout);

        assert_eq!(key_hex, key_hex_from_go);

        Ok(())
    }
}
