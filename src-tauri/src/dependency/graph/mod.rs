//! Module for generating and analyzing Rust dependency graphs.
//!
//! This module provides functionality to parse Cargo.toml and Cargo.lock files,
//! build a dependency graph, and perform various analyses on it.

mod builder;
mod edge;
mod export;
mod filter;
pub mod traversal;

use crate::dependency::{analysis::*, models::*, serialization::*};
use anyhow::{anyhow, Result};
use petgraph::graph::{DiGraph, NodeIndex};

pub use self::{
    builder::DependencyGraphBuilder,
    edge::DependencyEdge,
    export::{export_graph, ExportFormat},
    filter::DependencyFilter,
    traversal,
};
pub use crate::dependency::models::DependencyGraph;
