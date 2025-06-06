use fvm_shared4::bigint::BigInt;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::sector::{RegisteredPoStProof, RegisteredSealProof, StoragePower};
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

// A trait for runtime policy configuration
pub trait RuntimePolicy {
    fn policy(&self) -> &Policy;
}

// The policy itself
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Policy {
    //
    // --- miner policy ---
    //
    /// Maximum amount of sectors that can be aggregated.
    pub max_aggregated_sectors: u64,
    /// Minimum amount of sectors that can be aggregated.
    pub min_aggregated_sectors: u64,
    /// Maximum total aggregated proof size.
    pub max_aggregated_proof_size: usize,
    /// Maximum total replica update proof size.
    pub max_replica_update_proof_size: usize,

    /// The delay between pre commit expiration and clean up from state. This enforces that expired pre-commits
    /// stay in state for a period of time creating a grace period during which a late-running aggregated prove-commit
    /// can still prove its non-expired precommits without resubmitting a message
    pub expired_pre_commit_clean_up_delay: i64,

    /// The period over which all a miner's active sectors will be challenged.
    pub wpost_proving_period: ChainEpoch,
    /// The duration of a deadline's challenge window, the period before a deadline when the challenge is available.
    /// Notice that the challenge window is assumed to have the same duration as a deadline itself both in FIP and implementation.
    pub wpost_challenge_window: ChainEpoch,
    /// The number of non-overlapping PoSt deadlines in each proving period.
    pub wpost_period_deadlines: u64,
    /// The maximum distance back that a valid Window PoSt must commit to the current chain.
    pub wpost_max_chain_commit_age: ChainEpoch,
    /// WPoStDisputeWindow is the period after a Window PoSt window ends during which
    /// PoSts submitted during that period may be disputed.
    pub wpost_dispute_window: ChainEpoch,

    /// The maximum number of sectors that a miner can have simultaneously active.
    /// This also bounds the number of faults that can be declared, etc.
    pub sectors_max: usize,

    /// Maximum number of partitions that will be assigned to a deadline.
    pub max_partitions_per_deadline: u64,

    /// Maximum number of control addresses a miner may register.
    pub max_control_addresses: usize,

    /// MaxPeerIDLength is the maximum length allowed for any on-chain peer ID.
    pub max_peer_id_length: usize,

    /// MaxMultiaddrData is the maximum amount of data that can be stored in multiaddrs.
    pub max_multiaddr_data: usize,

    /// The maximum number of partitions that may be required to be loaded in a single invocation.
    /// This limits the number of simultaneous fault, recovery, or sector-extension declarations.
    pub addressed_partitions_max: u64,

    /// The maximum number of sector numbers addressable in a single invocation
    /// (which implies also the max infos that may be loaded at once).
    /// One upper bound on this is the max size of a storage block: 1MiB supports 130k at 8 bytes each,
    /// though bitfields can compress this.
    pub addressed_sectors_max: u64,

    /// The maximum number of partitions that can be proven in a single PoSt message.
    pub posted_partitions_max: u64,

    pub max_pre_commit_randomness_lookback: ChainEpoch,

    /// Number of epochs between publishing the precommit and when the challenge for interactive PoRep is drawn
    /// used to ensure it is not predictable by miner.
    pub pre_commit_challenge_delay: ChainEpoch,

    /// Maximum amount of sectors that can be aggregated in NI PoRep.
    pub max_aggregated_sectors_ni: u64,

    /// Minimum amount of sectors that can be aggregated.
    pub min_aggregated_sectors_ni: u64,

    /// Number of epochs between publishing the commit and when the randomness for non interactive PoRep is drawn
    pub max_prove_commit_ni_randomness_lookback: ChainEpoch,

    /// Allowed non interactive proof types for new miners
    pub valid_prove_commit_ni_proof_type: ProofSet,

    /// Lookback from the deadline's challenge window opening from which to sample chain randomness for the challenge seed.
    pub wpost_challenge_lookback: ChainEpoch,

    /// Minimum period before a deadline's challenge window opens that a fault must be declared for that deadline.
    pub fault_declaration_cutoff: ChainEpoch,

    /// The maximum age of a fault before the sector is terminated.
    pub fault_max_age: ChainEpoch,

    /// Staging period for a miner worker key change.
    pub worker_key_change_delay: ChainEpoch,

    /// Minimum number of epochs past the current epoch a sector may be set to expire.
    pub min_sector_expiration: i64,

    /// Maximum number of epochs past the current epoch a sector may be set to expire.
    /// The actual maximum extension will be the minimum of CurrEpoch + MaximumSectorExpirationExtension
    /// and sector.ActivationEpoch+sealProof.SectorMaximumLifetime()
    pub max_sector_expiration_extension: i64,

    /// Ratio of sector size to maximum deals per sector.
    /// The maximum number of deals is the sector size divided by this number.
    pub deal_limit_denominator: u64,

    /// Number of epochs after a consensus fault for which a miner is ineligible
    /// for permissioned actor methods and winning block elections.
    pub consensus_fault_ineligibility_duration: ChainEpoch,

    /// The maximum number of new sectors that may be staged by a miner during a single proving period.
    pub new_sectors_per_period_max: usize,

    /// Epochs after which chain state is final with overwhelming probability
    /// (hence the likelihood of two fork of this size is negligible).
    pub chain_finality: ChainEpoch,

    /// Allowed post proof types for new miners
    pub valid_post_proof_type: ProofSet,

    /// Allowed pre commit proof types for new miners
    pub valid_pre_commit_proof_type: ProofSet,

    /// Numerator of the fraction of circulating supply that will be used to calculate
    /// the daily fee for new sectors.
    pub daily_fee_circulating_supply_qap_multiplier_num: BigInt,
    /// Denominator of the fraction of circulating supply that will be used to calculate
    /// the daily fee for new sectors.
    pub daily_fee_circulating_supply_qap_multiplier_denom: BigInt,
    /// Denominator for the fraction of estimated daily block reward for the sector(s)
    /// attracting a fee, to be used as a cap for the fees when payable.
    /// No numerator is provided as the fee is calculated as a fraction of the estimated
    /// daily block reward.
    pub daily_fee_block_reward_cap_denom: i64,

    //
    // --- verifreg policy ---
    //
    /// Minimum verified deal size
    pub minimum_verified_allocation_size: StoragePower,
    /// Minimum term for a verified data allocation (epochs)
    pub minimum_verified_allocation_term: i64,
    /// Maximum term for a verified data allocaion (epochs)
    pub maximum_verified_allocation_term: i64,
    /// Maximum time a verified allocation can be active without being claimed (epochs).
    /// Supports recovery of erroneous allocations and prevents indefinite squatting on datacap.
    pub maximum_verified_allocation_expiration: i64,
    // Period of time at the end of a sector's life during which claims can be dropped
    pub end_of_life_claim_drop_period: ChainEpoch,

    //
    //  --- market policy ---
    //
    /// The number of blocks between payouts for deals
    pub deal_updates_interval: i64,

    /// Numerator of the percentage of normalized cirulating
    /// supply that must be covered by provider collateral
    pub prov_collateral_percent_supply_num: i64,

    /// Denominator of the percentage of normalized cirulating
    /// supply that must be covered by provider collateral
    pub prov_collateral_percent_supply_denom: i64,

    /// The default duration after a verified deal's nominal term to set for the corresponding
    /// allocation's maximum term.
    pub market_default_allocation_term_buffer: i64,

    //
    // --- power policy ---
    //
    /// Minimum miner consensus power
    pub minimum_consensus_power: StoragePower,
}

impl Default for Policy {
    fn default() -> Policy {
        Policy {
            max_aggregated_sectors: policy_constants::MAX_AGGREGATED_SECTORS,
            min_aggregated_sectors: policy_constants::MIN_AGGREGATED_SECTORS,
            max_aggregated_proof_size: policy_constants::MAX_AGGREGATED_PROOF_SIZE,
            max_replica_update_proof_size: policy_constants::MAX_REPLICA_UPDATE_PROOF_SIZE,
            expired_pre_commit_clean_up_delay: policy_constants::EXPIRED_PRE_COMMIT_CLEAN_UP_DELAY,
            wpost_proving_period: policy_constants::WPOST_PROVING_PERIOD,
            wpost_challenge_window: policy_constants::WPOST_CHALLENGE_WINDOW,
            wpost_period_deadlines: policy_constants::WPOST_PERIOD_DEADLINES,
            wpost_max_chain_commit_age: policy_constants::WPOST_MAX_CHAIN_COMMIT_AGE,
            wpost_dispute_window: policy_constants::WPOST_DISPUTE_WINDOW,
            sectors_max: policy_constants::SECTORS_MAX,
            max_partitions_per_deadline: policy_constants::MAX_PARTITIONS_PER_DEADLINE,
            max_control_addresses: policy_constants::MAX_CONTROL_ADDRESSES,
            max_peer_id_length: policy_constants::MAX_PEER_ID_LENGTH,
            max_multiaddr_data: policy_constants::MAX_MULTIADDR_DATA,
            addressed_partitions_max: policy_constants::ADDRESSED_PARTITIONS_MAX,
            addressed_sectors_max: policy_constants::ADDRESSED_SECTORS_MAX,
            posted_partitions_max: policy_constants::POSTED_PARTITIONS_MAX,
            max_pre_commit_randomness_lookback:
                policy_constants::MAX_PRE_COMMIT_RANDOMNESS_LOOKBACK,
            valid_prove_commit_ni_proof_type: ProofSet::default_seal_ni_proofs(),
            max_aggregated_sectors_ni: policy_constants::MAX_AGGREGATED_SECTORS_NI,
            min_aggregated_sectors_ni: policy_constants::MIN_AGGREGATED_SECTORS_NI,
            max_prove_commit_ni_randomness_lookback: policy_constants::MAX_PROVE_COMMIT_NI_LOOKBACK,
            pre_commit_challenge_delay: policy_constants::PRE_COMMIT_CHALLENGE_DELAY,
            wpost_challenge_lookback: policy_constants::WPOST_CHALLENGE_LOOKBACK,
            fault_declaration_cutoff: policy_constants::FAULT_DECLARATION_CUTOFF,
            fault_max_age: policy_constants::FAULT_MAX_AGE,
            worker_key_change_delay: policy_constants::WORKER_KEY_CHANGE_DELAY,
            min_sector_expiration: policy_constants::MIN_SECTOR_EXPIRATION,
            max_sector_expiration_extension: policy_constants::MAX_SECTOR_EXPIRATION_EXTENSION,
            deal_limit_denominator: policy_constants::DEAL_LIMIT_DENOMINATOR,
            consensus_fault_ineligibility_duration:
                policy_constants::CONSENSUS_FAULT_INELIGIBILITY_DURATION,
            new_sectors_per_period_max: policy_constants::NEW_SECTORS_PER_PERIOD_MAX,
            chain_finality: policy_constants::CHAIN_FINALITY,
            daily_fee_circulating_supply_qap_multiplier_num: BigInt::from_u64(
                policy_constants::DAILY_FEE_CIRCULATING_SUPPLY_QAP_MULTIPLIER_NUM,
            )
            .unwrap(),
            daily_fee_circulating_supply_qap_multiplier_denom: BigInt::from_u128(
                policy_constants::DAILY_FEE_CIRCULATING_SUPPLY_QAP_MULTIPLIER_DENOM,
            )
            .unwrap(),
            daily_fee_block_reward_cap_denom: policy_constants::DAILY_FEE_BLOCK_REWARD_CAP_DENOM,

            valid_post_proof_type: ProofSet::default_post_proofs(),
            valid_pre_commit_proof_type: ProofSet::default_precommit_seal_proofs(),
            minimum_verified_allocation_size: StoragePower::from_i32(
                policy_constants::MINIMUM_VERIFIED_ALLOCATION_SIZE,
            )
            .unwrap(),
            minimum_verified_allocation_term: policy_constants::MINIMUM_VERIFIED_ALLOCATION_TERM,
            maximum_verified_allocation_term: policy_constants::MAXIMUM_VERIFIED_ALLOCATION_TERM,
            maximum_verified_allocation_expiration:
                policy_constants::MAXIMUM_VERIFIED_ALLOCATION_EXPIRATION,
            end_of_life_claim_drop_period: policy_constants::END_OF_LIFE_CLAIM_DROP_PERIOD,
            deal_updates_interval: policy_constants::DEAL_UPDATES_INTERVAL,
            prov_collateral_percent_supply_num:
                policy_constants::PROV_COLLATERAL_PERCENT_SUPPLY_NUM,
            prov_collateral_percent_supply_denom:
                policy_constants::PROV_COLLATERAL_PERCENT_SUPPLY_DENOM,
            market_default_allocation_term_buffer:
                policy_constants::MARKET_DEFAULT_ALLOCATION_TERM_BUFFER,

            minimum_consensus_power: StoragePower::from(policy_constants::MINIMUM_CONSENSUS_POWER),
        }
    }
}

pub mod policy_constants {
    use fvm_shared4::clock::ChainEpoch;
    use fvm_shared4::sector::SectorNumber;

    use crate::v16::builtin::*;

    //
    // --- miner policy ---
    //

    /// The maximum assignable sector number.
    /// Raising this would require modifying our AMT implementation.
    pub const MAX_SECTOR_NUMBER: SectorNumber = i64::MAX as u64;

    // See comments on Policy struct.
    pub const MAX_AGGREGATED_SECTORS: u64 = 819;

    pub const MIN_AGGREGATED_SECTORS: u64 = 4;

    pub const MAX_AGGREGATED_PROOF_SIZE: usize = 81960;

    pub const MAX_REPLICA_UPDATE_PROOF_SIZE: usize = 4096;

    pub const EXPIRED_PRE_COMMIT_CLEAN_UP_DELAY: i64 = 8 * EPOCHS_IN_HOUR;

    pub const WPOST_PROVING_PERIOD: ChainEpoch = EPOCHS_IN_DAY;

    // Half an hour (=48 per day).
    // This must be consistent with WPOST_PERIOD_DEADLINES.
    pub const WPOST_CHALLENGE_WINDOW: ChainEpoch = 30 * 60 / EPOCH_DURATION_SECONDS;

    // This must be consistent with WPOST_CHALLENGE_WINDOW.
    pub const WPOST_PERIOD_DEADLINES: u64 = 48;

    pub const WPOST_MAX_CHAIN_COMMIT_AGE: ChainEpoch = WPOST_CHALLENGE_WINDOW;

    pub const WPOST_DISPUTE_WINDOW: ChainEpoch = 2 * CHAIN_FINALITY;

    pub const SECTORS_MAX: usize = 32 << 20;

    // For a minimum storage of upto 1Eib, we need 300 partitions per deadline.
    // 48 * 32GiB * 2349 * 300 = 1.00808144 EiB
    // So, to support upto 10Eib storage, we set this to 3000.
    pub const MAX_PARTITIONS_PER_DEADLINE: u64 = 3000;

    pub const MAX_CONTROL_ADDRESSES: usize = 10;

    // Most Peer IDs are expected to be less than 50 bytes.
    pub const MAX_PEER_ID_LENGTH: usize = 128;

    pub const MAX_MULTIADDR_DATA: usize = 1024;

    // With 48 deadlines (half-hour), 300 partitions per declaration permits addressing a full EiB
    // of partitions of 32GiB sectors with 1 message per epoch within a single half-hour deadline.
    // A miner can of course submit more messages.
    pub const ADDRESSED_PARTITIONS_MAX: u64 = MAX_PARTITIONS_PER_DEADLINE;

    pub const ADDRESSED_SECTORS_MAX: u64 = 25_000;

    pub const POSTED_PARTITIONS_MAX: u64 = 3;

    pub const MAX_PRE_COMMIT_RANDOMNESS_LOOKBACK: ChainEpoch = EPOCHS_IN_DAY + CHAIN_FINALITY;

    pub const PRE_COMMIT_CHALLENGE_DELAY: ChainEpoch = 150;

    // Maximum number of epochs within which to fetch a valid seal randomness from the chain for
    // a non-interactive PoRep proof. This balances the need to tie the seal to a particular chain with
    // but makes allowance for service providers to offer pre-sealed sectors within a larger window of
    // time.
    pub const MAX_PROVE_COMMIT_NI_LOOKBACK: ChainEpoch = 180 * EPOCHS_IN_DAY;

    pub const MAX_AGGREGATED_SECTORS_NI: u64 = 65;

    pub const MIN_AGGREGATED_SECTORS_NI: u64 = 1;

    // This lookback exists so that deadline windows can be non-overlapping (which make the programming simpler)
    // but without making the miner wait for chain stability before being able to start on PoSt computation.
    // The challenge is available this many epochs before the window is actually open to receiving a PoSt.
    pub const WPOST_CHALLENGE_LOOKBACK: ChainEpoch = 20;

    // This lookback must not be less than WPoStChallengeLookback lest a malicious miner be able to selectively declare
    // faults after learning the challenge value.
    pub const FAULT_DECLARATION_CUTOFF: ChainEpoch = WPOST_CHALLENGE_LOOKBACK + 50;

    pub const FAULT_MAX_AGE: ChainEpoch = WPOST_PROVING_PERIOD * 42;

    // Finality is a harsh delay for a miner who has lost their worker key, as the miner will miss Window PoSts until
    // it can be changed. It's the only safe value, though. We may implement a mitigation mechanism such as a second
    // key or allowing the owner account to submit PoSts while a key change is pending.
    pub const WORKER_KEY_CHANGE_DELAY: ChainEpoch = CHAIN_FINALITY;

    pub const MIN_SECTOR_EXPIRATION: i64 = 180 * EPOCHS_IN_DAY;

    pub const MAX_SECTOR_EXPIRATION_EXTENSION: i64 = 1278 * EPOCHS_IN_DAY;

    /// A value (2^27) limits 32GiB sectors to 256 deals and 64GiB sectors to 512.
    pub const DEAL_LIMIT_DENOMINATOR: u64 = 134217728;

    pub const CONSENSUS_FAULT_INELIGIBILITY_DURATION: ChainEpoch = CHAIN_FINALITY;

    pub const NEW_SECTORS_PER_PERIOD_MAX: usize = 128 << 10;

    /// This is a conservative value that is chosen via simulations of all known attacks.
    pub const CHAIN_FINALITY: ChainEpoch = 900;

    // Fraction of circulating supply per byte of quality adjusted power that will be used to calculate
    // the daily fee for new sectors.
    // The target multiplier is:
    //   5.56e-15 / 32GiB = 5.56e-15 / (32 * 2^30) = 5.56e-15 / 34,359,738,368 ≈ 1.61817e-25
    // (i.e. slightly rounded for simplicity and a more direct multiplication).
    // We implement this as 161817e-30.
    pub const DAILY_FEE_CIRCULATING_SUPPLY_QAP_MULTIPLIER_NUM: u64 = 161817;
    pub const DAILY_FEE_CIRCULATING_SUPPLY_QAP_MULTIPLIER_DENOM: u128 =
        1_000_000_000_000_000_000_000_000_000_000; // 10^30

    // 50% of estimated daily block rewards
    pub const DAILY_FEE_BLOCK_REWARD_CAP_DENOM: i64 = 2;

    //
    // --- verifreg policy ---
    //

    pub const MINIMUM_VERIFIED_ALLOCATION_SIZE: i32 = 1 << 20;
    pub const MINIMUM_VERIFIED_ALLOCATION_TERM: i64 = 180 * EPOCHS_IN_DAY;
    pub const MAXIMUM_VERIFIED_ALLOCATION_TERM: i64 = 5 * EPOCHS_IN_YEAR;
    pub const MAXIMUM_VERIFIED_ALLOCATION_EXPIRATION: i64 = 60 * EPOCHS_IN_DAY;
    pub const END_OF_LIFE_CLAIM_DROP_PERIOD: ChainEpoch = 30 * EPOCHS_IN_DAY;

    //
    // --- market policy ---
    //

    pub const DEAL_UPDATES_INTERVAL: i64 = 30 * EPOCHS_IN_DAY;

    pub const PROV_COLLATERAL_PERCENT_SUPPLY_NUM: i64 = 1;

    pub const PROV_COLLATERAL_PERCENT_SUPPLY_DENOM: i64 = 100;

    pub const MARKET_DEFAULT_ALLOCATION_TERM_BUFFER: i64 = 90 * EPOCHS_IN_DAY;

    //
    // --- power policy ---
    //

    pub const MINIMUM_CONSENSUS_POWER: i64 = 10 << 40;
}

/// A set indicating which proofs are considered valid, optimised for lookup of a small number of
/// sequential enum variants. Backed by an array of booleans where each index indicates if that
/// proof type is valid
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ProofSet(Vec<bool>);

/// The number of total possible types (enum variants) of RegisteredPoStProof
const REGISTERED_POST_PROOF_VARIANTS: usize = 15;

/// The number of total possible types (enum variants) of RegisteredSealProof
const REGISTERED_SEAL_PROOF_VARIANTS: usize = 20;

impl Default for ProofSet {
    fn default() -> Self {
        ProofSet(vec![
            false;
            REGISTERED_POST_PROOF_VARIANTS
                .max(REGISTERED_SEAL_PROOF_VARIANTS)
        ])
    }
}

impl ProofSet {
    /// Create a `ProofSet` for enabled `RegisteredPoStProof`s
    pub fn default_post_proofs() -> Self {
        let mut proofs = vec![false; REGISTERED_POST_PROOF_VARIANTS];

        proofs[i64::from(RegisteredPoStProof::StackedDRGWindow32GiBV1P1) as usize] = true;
        proofs[i64::from(RegisteredPoStProof::StackedDRGWindow64GiBV1P1) as usize] = true;

        ProofSet(proofs)
    }

    /// Create a `ProofSet` for enabled `RegisteredSealProof`s
    pub fn default_precommit_seal_proofs() -> Self {
        let mut proofs = vec![false; REGISTERED_SEAL_PROOF_VARIANTS];

        proofs[i64::from(RegisteredSealProof::StackedDRG32GiBV1P1) as usize] = true;
        proofs[i64::from(RegisteredSealProof::StackedDRG32GiBV1P1_Feat_SyntheticPoRep) as usize] =
            true;

        proofs[i64::from(RegisteredSealProof::StackedDRG64GiBV1P1) as usize] = true;
        proofs[i64::from(RegisteredSealProof::StackedDRG64GiBV1P1_Feat_SyntheticPoRep) as usize] =
            true;

        ProofSet(proofs)
    }

    pub fn default_seal_ni_proofs() -> Self {
        let mut proofs = vec![false; REGISTERED_SEAL_PROOF_VARIANTS];

        proofs[i64::from(RegisteredSealProof::StackedDRG32GiBV1P2_Feat_NiPoRep) as usize] = true;
        proofs[i64::from(RegisteredSealProof::StackedDRG64GiBV1P2_Feat_NiPoRep) as usize] = true;

        ProofSet(proofs)
    }

    /// Checks if the requested proof type exists in the set
    pub fn contains<P: Into<i64>>(&self, proof: P) -> bool {
        let index: i64 = proof.into();
        *self.0.get(index as usize).unwrap_or(&false)
    }

    /// Adds the requested proof type to the set of valid proofs
    pub fn insert<P: Into<i64>>(&mut self, proof: P) {
        let index: i64 = proof.into();
        self.0[index as usize] = true;
    }
}
