// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

#[cfg(test)]
mod tests {
    use fil_actor_interface::account::*;
    use fil_actor_interface::cron::*;
    use fil_actor_interface::datacap::*;
    use fil_actor_interface::ethaccount::*;
    use fil_actor_interface::evm::*;
    use fil_actor_interface::init::*;
    use fil_actor_interface::market::*;
    use fil_actor_interface::miner::*;
    use fil_actor_interface::multisig::*;
    use fil_actor_interface::placeholder::*;
    use fil_actor_interface::power::*;
    use fil_actor_interface::reward::*;
    use fil_actor_interface::system::*;
    use fil_actor_interface::verifreg::*;

    include!(concat!(env!("OUT_DIR"), "/network_manifest_tests.rs"));
}
