use cid::Cid;
use fvm_ipld_encoding::RawBytes;
use fvm_ipld_encoding::tuple::*;
use fvm_shared3::smooth::FilterEstimate;
use fvm_shared4::ActorID;
use fvm_shared4::bigint::{BigInt, bigint_ser};
use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::deal::DealID;
use fvm_shared4::econ::TokenAmount;
use fvm_shared4::piece::PaddedPieceSize;
use fvm_shared4::sector::SectorNumber;
use fvm_shared4::sector::{RegisteredSealProof, StoragePower};

use fil_actors_shared::v13::BatchReturn;

pub mod market {
    use super::*;
    use fvm_ipld_bitfield::BitField;

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct SectorDeals {
        pub sector_number: SectorNumber,
        pub sector_type: RegisteredSealProof,
        pub sector_expiry: ChainEpoch,
        pub deal_ids: Vec<DealID>,
    }

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct BatchActivateDealsParams {
        pub sectors: Vec<SectorDeals>,
        pub compute_cid: bool,
    }

    #[derive(Serialize_tuple, Deserialize_tuple, Clone)]
    pub struct ActivatedDeal {
        pub client: ActorID,
        pub allocation_id: u64,
        pub data: Cid,
        pub size: PaddedPieceSize,
    }

    #[derive(Serialize_tuple, Deserialize_tuple, Clone)]
    pub struct SectorDealActivation {
        pub activated: Vec<ActivatedDeal>,
        pub unsealed_cid: Option<Cid>,
    }

    #[derive(Serialize_tuple, Deserialize_tuple, Clone)]
    pub struct BatchActivateDealsResult {
        pub activation_results: BatchReturn,
        pub activations: Vec<SectorDealActivation>,
    }

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct OnMinerSectorsTerminateParams {
        pub epoch: ChainEpoch,
        pub sectors: BitField,
    }

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct SectorDataSpec {
        pub deal_ids: Vec<DealID>,
        pub sector_type: RegisteredSealProof,
    }

    #[derive(Serialize_tuple)]
    pub struct VerifyDealsForActivationParamsRef<'a> {
        pub sectors: &'a [SectorDeals],
    }

    #[derive(Serialize_tuple, Deserialize_tuple, Default, Clone)]
    pub struct VerifyDealsForActivationReturn {
        pub unsealed_cids: Vec<Option<Cid>>,
    }
}

pub mod power {
    use super::*;

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct CurrentTotalPowerReturn {
        #[serde(with = "bigint_ser")]
        pub raw_byte_power: StoragePower,
        #[serde(with = "bigint_ser")]
        pub quality_adj_power: StoragePower,
        pub pledge_collateral: TokenAmount,
        pub quality_adj_power_smoothed: FilterEstimate,
    }
    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct EnrollCronEventParams {
        pub event_epoch: ChainEpoch,
        pub payload: RawBytes,
    }

    #[derive(Serialize_tuple, Deserialize_tuple)]
    pub struct UpdateClaimedPowerParams {
        #[serde(with = "bigint_ser")]
        pub raw_byte_delta: StoragePower,
        #[serde(with = "bigint_ser")]
        pub quality_adjusted_delta: StoragePower,
    }
}

pub mod verifreg {
    use super::*;

    pub type ClaimID = u64;
    pub type AllocationID = u64;

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
        // The min period which the provider must commit to storing data
        pub term_min: ChainEpoch,
        // The max period for which provider can earn QA-power for the data
        pub term_max: ChainEpoch,
        // The epoch at which the (first range of the) piece was committed.
        pub term_start: ChainEpoch,
        // ID of the provider's sector in which the data is committed.
        pub sector: SectorNumber,
    }
    #[derive(Debug, Serialize_tuple, Deserialize_tuple)]
    pub struct GetClaimsParams {
        pub provider: ActorID,
        pub claim_ids: Vec<ClaimID>,
    }
    #[derive(Debug, Serialize_tuple, Deserialize_tuple)]

    pub struct GetClaimsReturn {
        pub batch_info: BatchReturn,
        pub claims: Vec<Claim>,
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
        pub sectors: Vec<SectorAllocationClaims>,
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
        pub sector_results: BatchReturn,
        pub sector_claims: Vec<SectorClaimSummary>,
    }
}
