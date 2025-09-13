//! Module for generating and analyzing Rust dependency graphs.
//!
//! This module provides functionality to parse Cargo.toml and Cargo.lock files,
//! build a dependency graph, and perform various analyses on it.

mod builder;
mod edge;
mod export;
mod filter;
pub mod traversal;

use anyhow::{anyhow, Result};
use petgraph::graph::{DiGraph, NodeIndex};

pub use self::builder::DependencyGraphBuilder;
pub use self::edge::DependencyEdge;
pub use self::export::{export_graph, ExportFormat};
pub use self::filter::DependencyFilter;
pub use self::traversal;
use crate::dependency::analysis::*;
pub use crate::dependency::models::DependencyGraph;
use crate::dependency::models::*;
use crate::dependency::serialization::*;
