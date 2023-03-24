// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

#[cfg(test)]
mod tests {
    use anyhow::*;
    use fil_actors_runtime_v10::runtime::Policy;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn policy_serde_roundtrip(policy: Policy) -> Result<()> {
        let serialized = toml::to_string(&policy)?;

        println!("serialized:\n{serialized}");

        let deserialized: Policy = toml::from_str(&serialized)?;

        ensure!(policy == deserialized);

        Ok(())
    }
}
