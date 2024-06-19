#!/bin/bash
set -euxo pipefail

crates=(
    "fil_actors_shared"
    "fil_actor_verifreg_state"
    "fil_actor_account_state"
    "fil_actor_cron_state"
    "fil_actor_datacap_state"
    "fil_actor_eam_state"
    "fil_actor_ethaccount_state"
    "fil_actor_evm_state"
    "fil_actor_init_state"
    "fil_actor_market_state"
    "fil_actor_miner_state"
    "fil_actor_multisig_state"
    "fil_actor_paych_state"
    "fil_actor_power_state"
    "fil_actor_reward_state"
    "fil_actor_system_state"
    "fil_actor_interface"
)

for crate in "${crates[@]}"; do
    # Publish to crates.io
    cargo publish --package "$crate" --token "$CRATES_IO_TOKEN"
done
