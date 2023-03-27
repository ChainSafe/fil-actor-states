// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime_v10::runtime::Policy as PolicyV10;
use fil_actors_runtime_v9::runtime::Policy as PolicyV9;
use fvm_shared::address::Address as AddressV2;
use fvm_shared::econ::TokenAmount as TokenAmountV2;
use fvm_shared::sector::RegisteredPoStProof as RegisteredPoStProofV2;
use fvm_shared::sector::RegisteredSealProof as RegisteredSealProofV2;
use fvm_shared::sector::SectorSize as SectorSizeV2;
use fvm_shared::smooth::FilterEstimate as FilterEstimateV2;
use fvm_shared3::address::Address as AddressV3;
use fvm_shared3::econ::TokenAmount as TokenAmountV3;
use fvm_shared3::sector::RegisteredPoStProof as RegisteredPoStProofV3;
use fvm_shared3::sector::RegisteredSealProof as RegisteredSealProofV3;
use fvm_shared3::sector::SectorSize as SectorSizeV3;
use fvm_shared3::smooth::FilterEstimate as FilterEstimateV3;

pub fn from_reg_seal_proof_v3_to_v2(proof: RegisteredSealProofV3) -> RegisteredSealProofV2 {
    let num_id: i64 = proof.into();
    RegisteredSealProofV2::from(num_id)
}

pub fn from_address_v2_to_v3(addr: AddressV2) -> AddressV3 {
    AddressV3::from_bytes(&addr.to_bytes())
        .expect("Couldn't convert between FVM2 and FVM3 addresses.")
}

pub fn from_address_v3_to_v2(addr: AddressV3) -> AddressV2 {
    AddressV2::from_bytes(&addr.to_bytes())
        .expect("Couldn't convert between FVM3 and FVM2 addresses.")
}

pub fn from_token_v3_to_v2(token: TokenAmountV3) -> TokenAmountV2 {
    TokenAmountV2::from_atto(token.atto().clone())
}

pub fn from_sector_size_v3_to_v2(proof: SectorSizeV3) -> SectorSizeV2 {
    match proof {
        SectorSizeV3::_2KiB => SectorSizeV2::_2KiB,
        SectorSizeV3::_8MiB => SectorSizeV2::_8MiB,
        SectorSizeV3::_512MiB => SectorSizeV2::_512MiB,
        SectorSizeV3::_32GiB => SectorSizeV2::_32GiB,
        SectorSizeV3::_64GiB => SectorSizeV2::_64GiB,
    }
}

pub fn from_reg_post_proof_v3_to_v2(proof: RegisteredPoStProofV3) -> RegisteredPoStProofV2 {
    let num_id: i64 = proof.into();
    RegisteredPoStProofV2::from(num_id)
}

pub fn from_filter_estimate_v3_to_v2(fe: FilterEstimateV3) -> FilterEstimateV2 {
    FilterEstimateV2 {
        position: fe.position,
        velocity: fe.velocity,
    }
}

pub fn from_policy_v10_to_v9(policy: &PolicyV10) -> PolicyV9 {
    PolicyV9 {
        max_aggregated_sectors: policy.max_aggregated_sectors,
        min_aggregated_sectors: policy.min_aggregated_sectors,
        max_aggregated_proof_size: policy.max_aggregated_proof_size,
        max_replica_update_proof_size: policy.max_replica_update_proof_size,
        pre_commit_sector_batch_max_size: policy.pre_commit_sector_batch_max_size,
        prove_replica_updates_max_size: policy.prove_replica_updates_max_size,
        expired_pre_commit_clean_up_delay: policy.expired_pre_commit_clean_up_delay,
        wpost_proving_period: policy.wpost_proving_period,
        wpost_challenge_window: policy.wpost_challenge_window,
        wpost_period_deadlines: policy.wpost_period_deadlines,
        wpost_max_chain_commit_age: policy.wpost_max_chain_commit_age,
        wpost_dispute_window: policy.wpost_dispute_window,
        sectors_max: policy.sectors_max,
        max_partitions_per_deadline: policy.max_partitions_per_deadline,
        max_control_addresses: policy.max_control_addresses,
        max_peer_id_length: policy.max_peer_id_length,
        max_multiaddr_data: policy.max_multiaddr_data,
        addressed_partitions_max: policy.addressed_partitions_max,
        declarations_max: policy.declarations_max,
        addressed_sectors_max: policy.addressed_sectors_max,
        max_pre_commit_randomness_lookback: policy.max_pre_commit_randomness_lookback,
        pre_commit_challenge_delay: policy.pre_commit_challenge_delay,
        wpost_challenge_lookback: policy.wpost_challenge_lookback,
        fault_declaration_cutoff: policy.fault_declaration_cutoff,
        fault_max_age: policy.fault_max_age,
        worker_key_change_delay: policy.worker_key_change_delay,
        min_sector_expiration: policy.min_sector_expiration,
        max_sector_expiration_extension: policy.max_sector_expiration_extension,
        deal_limit_denominator: policy.deal_limit_denominator,
        consensus_fault_ineligibility_duration: policy.consensus_fault_ineligibility_duration,
        new_sectors_per_period_max: policy.new_sectors_per_period_max,
        chain_finality: policy.chain_finality,
        valid_post_proof_type: policy
            .valid_post_proof_type
            .iter()
            .map(|proof| from_reg_post_proof_v3_to_v2(*proof))
            .collect(),
        valid_pre_commit_proof_type: policy
            .valid_pre_commit_proof_type
            .iter()
            .map(|proof| from_reg_seal_proof_v3_to_v2(*proof))
            .collect(),
        minimum_verified_allocation_size: policy.minimum_verified_allocation_size.clone(),
        minimum_verified_allocation_term: policy.minimum_verified_allocation_term,
        maximum_verified_allocation_term: policy.maximum_verified_allocation_term,
        maximum_verified_allocation_expiration: policy.maximum_verified_allocation_expiration,
        end_of_life_claim_drop_period: policy.end_of_life_claim_drop_period,
        deal_updates_interval: policy.deal_updates_interval,
        prov_collateral_percent_supply_num: policy.prov_collateral_percent_supply_num,
        prov_collateral_percent_supply_denom: policy.prov_collateral_percent_supply_denom,
        market_default_allocation_term_buffer: policy.market_default_allocation_term_buffer,
        minimum_consensus_power: policy.minimum_consensus_power.clone(),
    }
}