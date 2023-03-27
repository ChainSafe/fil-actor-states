// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared3::sector::StoragePower;

pub type AllocationID = u64;
pub type ClaimID = u64;

/// `DataCap` is an integer number of bytes.
/// We can introduce policy changes and replace this in the future.
pub type DataCap = StoragePower;
