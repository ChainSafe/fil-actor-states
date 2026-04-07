// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared4::clock::ChainEpoch;
use fvm_shared4::randomness::RANDOMNESS_LENGTH;
use num_derive::FromPrimitive;
use serde_repr::*;

/// Specifies a domain for randomness generation.
#[derive(PartialEq, Eq, Copy, Clone, FromPrimitive, Debug, Hash, Deserialize_repr)]
#[repr(i64)]
pub enum DomainSeparationTag {
    TicketProduction = 1,
    ElectionProofProduction = 2,
    WinningPoStChallengeSeed = 3,
    WindowedPoStChallengeSeed = 4,
    SealRandomness = 5,
    InteractiveSealChallengeSeed = 6,
    WindowPoStDeadlineAssignment = 7,
    MarketDealCronSeed = 8,
    PoStChainCommit = 9,
    EvmPrevRandao = 10,
}

#[allow(unused)]
pub fn draw_randomness(
    hasher: impl FnOnce(&[u8]) -> [u8; 32],
    rbase: &[u8; RANDOMNESS_LENGTH],
    pers: DomainSeparationTag,
    round: ChainEpoch,
    entropy: &[u8],
) -> [u8; RANDOMNESS_LENGTH] {
    let mut data = Vec::with_capacity(RANDOMNESS_LENGTH + 8 + 8 + entropy.len());

    // Append the personalization value
    let i64_bytes = (pers as i64).to_be_bytes();
    data.extend_from_slice(&i64_bytes);

    // Append the randomness
    data.extend_from_slice(rbase);

    // Append the round
    let i64_bytes = round.to_be_bytes();
    data.extend_from_slice(&i64_bytes);

    // Append the entropy
    data.extend_from_slice(entropy);

    hasher(&data)
    //
    // fvm::crypto::hash_blake2b(&data)
}
