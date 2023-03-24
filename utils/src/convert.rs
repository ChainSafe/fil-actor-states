// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

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

pub fn from_reg_seal_proof_v2_to_v3(proof: RegisteredSealProofV2) -> RegisteredSealProofV3 {
    let num_id: i64 = proof.into();
    RegisteredSealProofV3::from(num_id)
}

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
