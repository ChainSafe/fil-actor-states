// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_shared::actor_error_v12;
use fil_actors_shared::v12::runtime::Policy;
use fil_actors_shared::v12::{
    ActorError, AsActorError, Config, Map2, Multimap, DEFAULT_HAMT_CONFIG,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::RawBytes;
use fvm_ipld_hamt::BytesKey;
use fvm_shared3::smooth::FilterEstimate;
use fvm_shared4::address::Address;
use fvm_shared4::bigint::bigint_ser;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::error::ExitCode;
use fvm_shared4::sector::{RegisteredPoStProof, StoragePower};
use fvm_shared4::ActorID;
use integer_encoding::VarInt;
use lazy_static::lazy_static;
use num_traits::Signed;

use super::CONSENSUS_MINER_MIN_MINERS;

lazy_static! {
    /// genesis power in bytes = 750,000 GiB
    pub static ref INITIAL_QA_POWER_ESTIMATE_POSITION: StoragePower = StoragePower::from(750_000) * (1 << 30);
    /// max chain throughput in bytes per epoch = 120 ProveCommits / epoch = 3,840 GiB
    pub static ref INITIAL_QA_POWER_ESTIMATE_VELOCITY: StoragePower = StoragePower::from(3_840) * (1 << 30);
}

pub const CRON_QUEUE_HAMT_BITWIDTH: u32 = 6;
pub const CRON_QUEUE_AMT_BITWIDTH: u32 = 6;
pub const PROOF_VALIDATION_BATCH_AMT_BITWIDTH: u32 = 4;

pub type ClaimsMap<BS> = Map2<BS, Address, Claim>;
pub const CLAIMS_CONFIG: Config = DEFAULT_HAMT_CONFIG;

/// Storage power actor state
#[derive(Default, Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct State {
    #[serde(with = "bigint_ser")]
    pub total_raw_byte_power: StoragePower,
    #[serde(with = "bigint_ser")]
    pub total_bytes_committed: StoragePower,
    #[serde(with = "bigint_ser")]
    pub total_quality_adj_power: StoragePower,
    #[serde(with = "bigint_ser")]
    pub total_qa_bytes_committed: StoragePower,
    pub total_pledge_collateral: TokenAmount,

    #[serde(with = "bigint_ser")]
    pub this_epoch_raw_byte_power: StoragePower,
    #[serde(with = "bigint_ser")]
    pub this_epoch_quality_adj_power: StoragePower,
    pub this_epoch_pledge_collateral: TokenAmount,
    pub this_epoch_qa_power_smoothed: FilterEstimate,

    pub miner_count: i64,
    /// Number of miners having proven the minimum consensus power.
    pub miner_above_min_power_count: i64,

    /// A queue of events to be triggered by cron, indexed by epoch.
    pub cron_event_queue: Cid, // Multimap, (HAMT[ChainEpoch]AMT[CronEvent]

    /// First epoch in which a cron task may be stored. Cron will iterate every epoch between this
    /// and the current epoch inclusively to find tasks to execute.
    pub first_cron_epoch: ChainEpoch,

    /// Claimed power for each miner.
    pub claims: Cid, // Map, HAMT[address]Claim

    pub proof_validation_batch: Option<Cid>,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<State> {
        let empty_claims = ClaimsMap::empty(store, CLAIMS_CONFIG, "empty").flush()?;
        let empty_mmap = Multimap::new(store, CRON_QUEUE_HAMT_BITWIDTH, CRON_QUEUE_AMT_BITWIDTH)
            .root()
            .context_code(
                ExitCode::USR_ILLEGAL_STATE,
                "Failed to get empty multimap cid",
            )?;
        Ok(State {
            cron_event_queue: empty_mmap,
            claims: empty_claims,
            this_epoch_qa_power_smoothed: FilterEstimate::new(
                INITIAL_QA_POWER_ESTIMATE_POSITION.clone(),
                INITIAL_QA_POWER_ESTIMATE_VELOCITY.clone(),
            ),
            ..Default::default()
        })
    }

    pub fn into_total_locked(self) -> TokenAmount {
        self.total_pledge_collateral
    }

    /// Checks power actor state for if miner meets minimum consensus power.
    pub fn miner_nominal_power_meets_consensus_minimum<BS: Blockstore>(
        &self,
        policy: &Policy,
        s: &BS,
        miner: ActorID,
    ) -> Result<(StoragePower, bool), ActorError> {
        let claims = self.load_claims(s)?;
        let a = &Address::new_id(miner);
        let claim = claims
            .get(a)?
            .with_context_code(ExitCode::USR_ILLEGAL_ARGUMENT, || {
                format!("no claim for actor: {}", miner)
            })?;

        let miner_nominal_power = claim.raw_byte_power.clone();
        let miner_min_power = consensus_miner_min_power(policy, claim.window_post_proof_type)
            .context_code(
                ExitCode::USR_ILLEGAL_STATE,
                "could not get miner min power from proof type: {}",
            )?;

        if miner_nominal_power >= miner_min_power {
            // If miner is larger than min power requirement, valid
            Ok((miner_nominal_power, true))
        } else if self.miner_above_min_power_count >= CONSENSUS_MINER_MIN_MINERS {
            // if min consensus miners requirement met, return false
            Ok((miner_nominal_power, false))
        } else {
            // if fewer miners than consensus minimum, return true if non-zero power
            Ok((
                miner_nominal_power.clone(),
                miner_nominal_power.is_positive(),
            ))
        }
    }

    pub fn miner_power<BS: Blockstore>(
        &self,
        s: &BS,
        miner: &Address,
    ) -> Result<Option<Claim>, ActorError> {
        let claims = self.load_claims(s)?;
        claims.get(miner).map(|s| s.cloned())
    }

    pub fn load_claims<BS: Blockstore>(&self, s: BS) -> Result<ClaimsMap<BS>, ActorError> {
        ClaimsMap::load(s, &self.claims, CLAIMS_CONFIG, "claims")
    }

    pub fn save_claims<BS: Blockstore>(
        &mut self,
        claims: &mut ClaimsMap<BS>,
    ) -> Result<(), ActorError> {
        self.claims = claims.flush()?;
        Ok(())
    }

    pub fn current_total_power(&self) -> (StoragePower, StoragePower) {
        if self.miner_above_min_power_count < CONSENSUS_MINER_MIN_MINERS {
            (
                self.total_bytes_committed.clone(),
                self.total_qa_bytes_committed.clone(),
            )
        } else {
            (
                self.total_raw_byte_power.clone(),
                self.total_quality_adj_power.clone(),
            )
        }
    }

    pub fn get_claim<BS: Blockstore>(
        &self,
        store: &BS,
        miner: &Address,
    ) -> anyhow::Result<Option<Claim>> {
        let claims = self.load_claims(store)?;
        let claim = claims.get(miner)?;
        Ok(claim.cloned())
    }
}

pub fn set_claim<BS: Blockstore>(
    claims: &mut ClaimsMap<BS>,
    a: &Address,
    claim: Claim,
) -> Result<(), ActorError> {
    if claim.raw_byte_power.is_negative() {
        return Err(actor_error_v12!(
            illegal_state,
            "negative claim raw power {}",
            claim.raw_byte_power
        ));
    }
    if claim.quality_adj_power.is_negative() {
        return Err(actor_error_v12!(
            illegal_state,
            "negative claim quality-adjusted power {}",
            claim.quality_adj_power
        ));
    }

    claims.set(a, claim)?;
    Ok(())
}

pub fn epoch_key(e: ChainEpoch) -> BytesKey {
    let bz = e.encode_var_vec();
    bz.into()
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple, Clone, PartialEq, Eq)]
pub struct Claim {
    /// Miner's proof type used to determine minimum miner size
    pub window_post_proof_type: RegisteredPoStProof,
    /// Sum of raw byte power for a miner's sectors.
    #[serde(with = "bigint_ser")]
    pub raw_byte_power: StoragePower,
    /// Sum of quality adjusted power for a miner's sectors.
    #[serde(with = "bigint_ser")]
    pub quality_adj_power: StoragePower,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CronEvent {
    pub miner_addr: Address,
    pub callback_payload: RawBytes,
}

/// Returns the minimum storage power required for each PoSt proof type.
pub fn consensus_miner_min_power(
    policy: &Policy,
    p: RegisteredPoStProof,
) -> anyhow::Result<StoragePower> {
    use RegisteredPoStProof::*;
    match p {
        StackedDRGWinning2KiBV1
        | StackedDRGWinning8MiBV1
        | StackedDRGWinning512MiBV1
        | StackedDRGWinning32GiBV1
        | StackedDRGWinning64GiBV1
        | StackedDRGWindow2KiBV1P1
        | StackedDRGWindow8MiBV1P1
        | StackedDRGWindow512MiBV1P1
        | StackedDRGWindow32GiBV1P1
        | StackedDRGWindow64GiBV1P1 => Ok(policy.minimum_consensus_power.clone()),
        Invalid(i) => Err(anyhow::anyhow!("unsupported proof type: {}", i)),
    }
}

#[cfg(test)]
mod test {
    use fvm_shared4::clock::ChainEpoch;

    use super::*;

    #[test]
    fn epoch_key_test() {
        let e1: ChainEpoch = 101;
        let e2: ChainEpoch = 102;
        let e3: ChainEpoch = 103;
        let e4: ChainEpoch = -1;

        let b1: BytesKey = [0xca, 0x1].to_vec().into();
        let b2: BytesKey = [0xcc, 0x1].to_vec().into();
        let b3: BytesKey = [0xce, 0x1].to_vec().into();
        let b4: BytesKey = [0x1].to_vec().into();

        assert_eq!(b1, epoch_key(e1));
        assert_eq!(b2, epoch_key(e2));
        assert_eq!(b3, epoch_key(e3));
        assert_eq!(b4, epoch_key(e4));
    }
}
