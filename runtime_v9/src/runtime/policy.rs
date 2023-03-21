// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub use fil_actors_runtime_v10::runtime::policy_constants;
pub use fil_actors_runtime_v10::runtime::Policy;

// A trait for runtime policy configuration
pub trait RuntimePolicy {
    fn policy(&self) -> &Policy;
}
