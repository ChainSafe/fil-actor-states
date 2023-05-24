// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_hamt::BytesKey;
use fvm_shared3::address::Address;

pub use self::batch_return::BatchReturn;
pub use self::batch_return::BatchReturnGen;
pub use self::batch_return::FailCode;
pub use self::downcast::*;
pub use self::mapmap::MapMap;
pub use self::multimap::*;
pub use self::set::Set;
pub use self::set_multimap::SetMultimap;

mod batch_return;
mod downcast;
mod mapmap;
mod multimap;
mod set;
mod set_multimap;

/// Converts address key to key, equivilent go code can be found at L54-L61 in
/// <https://github.com/filecoin-project/go-state-types/blob/master/builtin/v9/migration/datacap.go#L54>
pub fn hamt_addr_key_to_key(addr_key: &BytesKey) -> anyhow::Result<BytesKey> {
    let addr = Address::from_bytes(addr_key)?;
    Ok(addr.payload_bytes().into())
}

#[cfg(test)]
mod tests {
    use std::{array::from_fn, process::Command};

    use super::*;
    use anyhow::*;
    use fil_actors_test_utils::go_compat::{ensure_go_mod_prepared, go_compat_tests_dir};
    use fvm_shared3::address::{Protocol, MAX_SUBADDRESS_LEN};
    use pretty_assertions::assert_eq;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    #[derive(Debug, Clone)]
    struct AddressWrapper(Address);

    impl Arbitrary for AddressWrapper {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut addr = Address::arbitrary(g);
            // HACK: Go code cannot parse uint greater than u63 upper bound
            match addr.protocol() {
                Protocol::ID => {
                    addr = Address::new_id(u32::arbitrary(g) as _);
                }
                Protocol::Delegated => {
                    let buffer: [u8; MAX_SUBADDRESS_LEN] = from_fn(|_| u8::arbitrary(g));
                    let length = usize::arbitrary(g) % (MAX_SUBADDRESS_LEN + 1);
                    addr =
                        Address::new_delegated(u32::arbitrary(g) as _, &buffer[..length]).unwrap();
                }
                _ => {}
            }

            Self(addr)
        }
    }

    #[quickcheck]
    fn test_hamt_addr_key_to_key(addr: AddressWrapper) -> Result<()> {
        ensure_go_mod_prepared();

        let addr = addr.0;
        let addr_key: BytesKey = addr.to_bytes().into();
        let addr_key_hex = hex::encode(&addr_key.0);
        let key = hamt_addr_key_to_key(&addr_key)?;
        let key_hex = hex::encode(&key.0);

        let app = Command::new("go")
            .args([
                "run",
                "shared/v9/test_addr_key_to_key.go",
                "--addr",
                addr_key_hex.as_str(),
            ])
            .current_dir(go_compat_tests_dir()?)
            .output()?;

        if !app.stderr.is_empty() {
            println!("{}", String::from_utf8_lossy(&app.stderr));
            anyhow::bail!("Fail to run go test");
        }

        let key_hex_from_go = String::from_utf8_lossy(&app.stdout);

        assert_eq!(key_hex, key_hex_from_go);

        Ok(())
    }
}
