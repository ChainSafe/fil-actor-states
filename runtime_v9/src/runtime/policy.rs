pub use fil_actors_runtime_v10::runtime::Policy;
pub use fil_actors_runtime_v10::runtime::policy_constants;

// A trait for runtime policy configuration
pub trait RuntimePolicy {
    fn policy(&self) -> &Policy;
}
