// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_shared::actor_error_v13;
use fil_actors_shared::v13::{ActorError, AsActorError, Config, DEFAULT_HAMT_CONFIG, Map2, MapMap};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared4::address::Address;
use fvm_shared4::bigint::bigint_ser::BigIntDe;
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::error::ExitCode;
use fvm_shared4::piece::PaddedPieceSize;
use fvm_shared4::sector::SectorNumber;
use fvm_shared4::{ActorID, HAMT_BIT_WIDTH};

use crate::v13::{AddrPairKey, AllocationID, ClaimID};
use crate::v13::{DataCap, RemoveDataCapProposalID};

pub type DataCapMap<BS> = Map2<BS, Address, BigIntDe>;
pub const DATACAP_MAP_CONFIG: Config = DEFAULT_HAMT_CONFIG;

pub type RemoveDataCapProposalMap<BS> = Map2<BS, AddrPairKey, RemoveDataCapProposalID>;
pub const REMOVE_DATACAP_PROPOSALS_CONFIG: Config = DEFAULT_HAMT_CONFIG;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    pub root_key: Address,
    // Maps verifier addresses to data cap minting allowance (in bytes).
    pub verifiers: Cid, // HAMT[Address]DataCap
    pub remove_data_cap_proposal_ids: Cid,
    // Maps client IDs to allocations made by that client.
    pub allocations: Cid, // HAMT[ActorID]HAMT[AllocationID]Allocation
    // Next allocation identifier to use.
    // The value 0 is reserved to mean "no allocation".
    pub next_allocation_id: u64,
    // Maps provider IDs to allocations claimed by that provider.
    pub claims: Cid, // HAMT[ActorID]HAMT[ClaimID]Claim
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS, root_key: Address) -> Result<State, ActorError> {
        let empty_dcap = DataCapMap::empty(store, DATACAP_MAP_CONFIG, "empty").flush()?;
        let empty_allocs_claims =
            MapMap::<_, (), ActorID, u64>::new(store, HAMT_BIT_WIDTH, HAMT_BIT_WIDTH)
                .flush()
                .map_err(|e| {
                    actor_error_v13!(illegal_state, "failed to create empty multi map: {}", e)
                })?;

        Ok(State {
            root_key,
            verifiers: empty_dcap,
            remove_data_cap_proposal_ids: empty_dcap,
            allocations: empty_allocs_claims,
            next_allocation_id: 1,
            claims: empty_allocs_claims,
        })
    }

    // Adds a verifier and cap, overwriting any existing cap for that verifier.
    pub fn put_verifier(
        &mut self,
        store: &impl Blockstore,
        verifier: &Address,
        cap: &DataCap,
    ) -> Result<(), ActorError> {
        let mut verifiers = self.load_verifiers(store)?;
        verifiers.set(verifier, BigIntDe(cap.clone()))?;
        self.verifiers = verifiers.flush()?;
        Ok(())
    }

    pub fn remove_verifier(
        &mut self,
        store: &impl Blockstore,
        verifier: &Address,
    ) -> Result<(), ActorError> {
        let mut verifiers = self.load_verifiers(store)?;
        verifiers
            .delete(verifier)?
            .context_code(ExitCode::USR_ILLEGAL_ARGUMENT, "verifier not found")?;
        self.verifiers = verifiers.flush()?;
        Ok(())
    }

    pub fn get_verifier_cap(
        &self,
        store: &impl Blockstore,
        verifier: &Address,
    ) -> Result<Option<DataCap>, ActorError> {
        let verifiers = self.load_verifiers(store)?;
        let allowance = verifiers.get(verifier)?;
        Ok(allowance.map(|a| a.clone().0))
    }

    pub fn load_verifiers<BS: Blockstore>(&self, store: BS) -> Result<DataCapMap<BS>, ActorError> {
        DataCapMap::load(store, &self.verifiers, DATACAP_MAP_CONFIG, "verifiers")
    }

    pub fn load_allocs<'a, BS: Blockstore>(
        &self,
        store: &'a BS,
    ) -> Result<MapMap<'a, BS, Allocation, ActorID, AllocationID>, ActorError> {
        MapMap::<BS, Allocation, ActorID, AllocationID>::from_root(
            store,
            &self.allocations,
            HAMT_BIT_WIDTH,
            HAMT_BIT_WIDTH,
        )
        .context_code(
            ExitCode::USR_ILLEGAL_STATE,
            "failed to load allocations table",
        )
    }

    pub fn save_allocs<BS: Blockstore>(
        &mut self,
        allocs: &mut MapMap<'_, BS, Allocation, ActorID, AllocationID>,
    ) -> Result<(), ActorError> {
        self.allocations = allocs.flush().context_code(
            ExitCode::USR_ILLEGAL_STATE,
            "failed to flush allocations table",
        )?;
        Ok(())
    }

    /// Inserts a batch of allocations under a single client address.
    /// The allocations are assigned sequential IDs starting from the next available.
    pub fn insert_allocations<BS: Blockstore>(
        &mut self,
        store: &BS,
        client: ActorID,
        new_allocs: Vec<Allocation>,
    ) -> Result<Vec<AllocationID>, ActorError> {
        if new_allocs.is_empty() {
            return Ok(vec![]);
        }
        let mut allocs = self.load_allocs(store)?;
        // These local variables allow the id-associating map closure to move the allocations
        // from the iterator rather than clone, without moving self.
        let first_id = self.next_allocation_id;
        let mut count = 0;
        let count_ref = &mut count;
        allocs
            .put_many(
                client,
                new_allocs.into_iter().map(move |a| {
                    let id = first_id + *count_ref;
                    *count_ref += 1;
                    (id, a)
                }),
            )
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to put allocations")?;
        self.save_allocs(&mut allocs)?;
        self.next_allocation_id += count;
        let allocated_ids = (first_id..first_id + count).collect();
        Ok(allocated_ids)
    }

    pub fn load_claims<'a, BS: Blockstore>(
        &self,
        store: &'a BS,
    ) -> Result<MapMap<'a, BS, Claim, ActorID, ClaimID>, ActorError> {
        MapMap::<BS, Claim, ActorID, ClaimID>::from_root(
            store,
            &self.claims,
            HAMT_BIT_WIDTH,
            HAMT_BIT_WIDTH,
        )
        .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to load claims table")
    }

    pub fn save_claims<BS: Blockstore>(
        &mut self,
        claims: &mut MapMap<'_, BS, Claim, ActorID, ClaimID>,
    ) -> Result<(), ActorError> {
        self.claims = claims
            .flush()
            .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to flush claims table")?;
        Ok(())
    }

    pub fn put_claims<BS: Blockstore>(
        &mut self,
        store: &BS,
        claims: Vec<(ClaimID, Claim)>,
    ) -> Result<(), ActorError> {
        if claims.is_empty() {
            return Ok(());
        }
        let mut st_claims = self.load_claims(store)?;
        for (id, claim) in claims.into_iter() {
            st_claims
                .put(claim.provider, id, claim)
                .context_code(ExitCode::USR_ILLEGAL_STATE, "failed to put claim")?;
        }
        self.save_claims(&mut st_claims)?;
        Ok(())
    }
}
#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    // The provider storing the data (from allocation).
    pub provider: ActorID,
    // The client which allocated the DataCap (from allocation).
    pub client: ActorID,
    // Identifier of the data committed (from allocation).
    pub data: Cid,
    // The (padded) size of data (from allocation).
    pub size: PaddedPieceSize,
    // The min period after term_start which the provider must commit to storing data
    pub term_min: ChainEpoch,
    // The max period after term_start for which provider can earn QA-power for the data
    pub term_max: ChainEpoch,
    // The epoch at which the (first range of the) piece was committed.
    pub term_start: ChainEpoch,
    // ID of the provider's sector in which the data is committed.
    pub sector: SectorNumber,
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, PartialEq, Eq)]
pub struct Allocation {
    // The verified client which allocated the DataCap.
    pub client: ActorID,
    // The provider (miner actor) which may claim the allocation.
    pub provider: ActorID,
    // Identifier of the data to be committed.
    pub data: Cid,
    // The (padded) size of data.
    pub size: PaddedPieceSize,
    // The minimum duration which the provider must commit to storing the piece to avoid
    // early-termination penalties (epochs).
    pub term_min: ChainEpoch,
    // The maximum period for which a provider can earn quality-adjusted power
    // for the piece (epochs).
    pub term_max: ChainEpoch,
    // The latest epoch by which a provider must commit data before the allocation expires.
    pub expiration: ChainEpoch,
}

pub fn get_allocation<'a, BS>(
    allocations: &'a mut MapMap<BS, Allocation, ActorID, AllocationID>,
    client: ActorID,
    id: AllocationID,
) -> Result<Option<&'a Allocation>, ActorError>
where
    BS: Blockstore,
{
    allocations.get(client, id).context_code(
        ExitCode::USR_ILLEGAL_STATE,
        "HAMT lookup failure getting allocation",
    )
}

pub fn get_claim<'a, BS>(
    claims: &'a mut MapMap<BS, Claim, ActorID, ClaimID>,
    provider: ActorID,
    id: ClaimID,
) -> Result<Option<&'a Claim>, ActorError>
where
    BS: Blockstore,
{
    claims.get(provider, id).context_code(
        ExitCode::USR_ILLEGAL_STATE,
        "HAMT lookup failure getting claim",
    )
}
