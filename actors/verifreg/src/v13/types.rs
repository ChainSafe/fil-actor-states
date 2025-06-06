// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_shared::v13::{BatchReturn, MapKey};
use fvm_ipld_encoding::tuple::*;
use fvm_shared4::ActorID;
use fvm_shared4::address::Address;
use fvm_shared4::bigint::{BigInt, bigint_ser};
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::crypto::signature::Signature;
use fvm_shared4::piece::PaddedPieceSize;
use fvm_shared4::sector::SectorNumber;
use fvm_shared4::sector::StoragePower;
use std::fmt::{Debug, Formatter};

use crate::v13::Claim;

pub type AllocationID = u64;
pub type ClaimID = u64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct ConstructorParams {
    pub root_key: Address,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct VerifierParams {
    pub address: Address,
    #[serde(with = "bigint_ser")]
    pub allowance: DataCap,
}

pub type AddVerifierParams = VerifierParams;

pub type AddVerifiedClientParams = VerifierParams;

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct RemoveVerifierParams {
    pub verifier: Address,
}

/// DataCap is an integer number of bytes.
/// We can introduce policy changes and replace this in the future.
pub type DataCap = StoragePower;

pub const SIGNATURE_DOMAIN_SEPARATION_REMOVE_DATA_CAP: &[u8] = b"fil_removedatacap:";

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveDataCapParams {
    pub verified_client_to_remove: Address,
    #[serde(with = "bigint_ser")]
    pub data_cap_amount_to_remove: DataCap,
    pub verifier_request_1: RemoveDataCapRequest,
    pub verifier_request_2: RemoveDataCapRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveDataCapRequest {
    pub verifier: Address,
    pub signature: Signature,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveDataCapReturn {
    pub verified_client: Address,
    #[serde(with = "bigint_ser")]
    pub data_cap_removed: DataCap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveDataCapProposalID {
    pub id: u64,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveDataCapProposal {
    pub verified_client: Address,
    #[serde(with = "bigint_ser")]
    pub data_cap_amount: DataCap,
    pub removal_proposal_id: RemoveDataCapProposalID,
}

pub struct AddrPairKey {
    pub first: Address,
    pub second: Address,
}

impl AddrPairKey {
    pub fn new(first: Address, second: Address) -> Self {
        AddrPairKey { first, second }
    }
}

impl Debug for AddrPairKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self.first, self.second).fmt(f)
    }
}

impl MapKey for AddrPairKey {
    fn from_bytes(_b: &[u8]) -> Result<Self, String> {
        unimplemented!()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut first = self.first.to_bytes();
        let mut second = self.second.to_bytes();
        first.append(&mut second);
        Ok(first)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveExpiredAllocationsParams {
    // Client for which to remove expired allocations.
    pub client: ActorID,
    // Optional list of allocation IDs to attempt to remove.
    // Empty means remove all eligible expired allocations.
    pub allocation_ids: Vec<AllocationID>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveExpiredAllocationsReturn {
    // Ids of the allocations that were either specified by the caller or discovered to be expired.
    pub considered: Vec<AllocationID>,
    // Results for each processed allocation.
    pub results: BatchReturn,
    // The amount of datacap reclaimed for the client.
    #[serde(with = "bigint_ser")]
    pub datacap_recovered: DataCap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct SectorAllocationClaims {
    pub sector: SectorNumber,
    pub expiry: ChainEpoch,
    pub claims: Vec<AllocationClaim>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct AllocationClaim {
    pub client: ActorID,
    pub allocation_id: AllocationID,
    pub data: Cid,
    pub size: PaddedPieceSize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ClaimAllocationsParams {
    /// Allocations to claim, grouped by sector.
    pub sectors: Vec<SectorAllocationClaims>,
    /// Whether to abort entirely if any claim fails.
    /// If false, a failed claim will cause other claims in the same sector group to also fail,
    /// but allow other sectors to proceed.
    pub all_or_nothing: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct SectorClaimSummary {
    #[serde(with = "bigint_ser")]
    pub claimed_space: BigInt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ClaimAllocationsReturn {
    /// Status of each sector grouping of claims.
    pub sector_results: BatchReturn,
    /// The claimed space for each successful sector group.
    pub sector_claims: Vec<SectorClaimSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ClaimTerm {
    pub provider: ActorID,
    pub claim_id: ClaimID,
    pub term_max: ChainEpoch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ExtendClaimTermsParams {
    pub terms: Vec<ClaimTerm>,
}

pub type ExtendClaimTermsReturn = BatchReturn;

//
// Receiver hook payload
//

// A request to create an allocation with datacap tokens.
// See Allocation state for description of field semantics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct AllocationRequest {
    pub provider: ActorID,
    pub data: Cid,
    pub size: PaddedPieceSize,
    pub term_min: ChainEpoch,
    pub term_max: ChainEpoch,
    pub expiration: ChainEpoch,
}

// A request to extend the term of an existing claim with datacap tokens.
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ClaimExtensionRequest {
    pub provider: ActorID,
    pub claim: ClaimID,
    pub term_max: ChainEpoch,
}

/// Operator-data payload for a datacap token transfer receiver hook specifying an allocation.
/// The implied client is the sender of the datacap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct AllocationRequests {
    pub allocations: Vec<AllocationRequest>,
    pub extensions: Vec<ClaimExtensionRequest>,
}

/// Recipient data payload in response to a datacap token transfer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct AllocationsResponse {
    // Result for each allocation request.
    pub allocation_results: BatchReturn,
    // Result for each extension request.
    pub extension_results: BatchReturn,
    // IDs of new allocations created.
    pub new_allocations: Vec<AllocationID>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct GetClaimsParams {
    pub provider: ActorID,
    pub claim_ids: Vec<ClaimID>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct GetClaimsReturn {
    pub batch_info: BatchReturn,
    pub claims: Vec<Claim>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveExpiredClaimsParams {
    // Provider to clean up (need not be the caller)
    pub provider: ActorID,
    // Optional list of claim IDs to attempt to remove.
    // Empty means remove all eligible expired claims.
    pub claim_ids: Vec<ClaimID>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct RemoveExpiredClaimsReturn {
    // Ids of the claims that were either specified by the caller or discovered to be expired.
    pub considered: Vec<AllocationID>,
    // Results for each processed claim.
    pub results: BatchReturn,
}
