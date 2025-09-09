//! Module for generating and analyzing Rust dependency graphs.
//! 
//! This module provides functionality to parse Cargo.toml and Cargo.lock files,
//! build a dependency graph, and perform various analyses on it.

mod builder;
mod edge;
pub mod traversal;
mod filter;
mod export;

use crate::dependency::{
    models::*,
    analysis::*,
    serialization::*,
};
use petgraph::graph::{DiGraph, NodeIndex};
use anyhow::{anyhow, Result};

pub use self::{
    builder::DependencyGraphBuilder,
    edge::DependencyEdge,
    filter::DependencyFilter,
    export::{ExportFormat, export_graph},
    traversal,
};
pub use crate::dependency::models::DependencyGraph;


