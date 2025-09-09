//! Integration tests for architectural analysis

// Re-export test helpers
pub use super::test_helpers::*;

// Import test modules
mod circular_dependencies;
mod dependency_inversion;
mod interface_segregation;
mod layer_violations;

// Re-export test modules for use in integration tests
pub use circular_dependencies::*;
pub use dependency_inversion::*;
pub use interface_segregation::*;
pub use layer_violations::*;
