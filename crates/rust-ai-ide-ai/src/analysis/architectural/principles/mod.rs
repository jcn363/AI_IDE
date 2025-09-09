//! Implementation of SOLID principles analysis

mod dependency_inversion;
mod interface_segregation;

pub use dependency_inversion::DependencyInversionAnalyzer;
pub use interface_segregation::InterfaceSegregationAnalyzer;
