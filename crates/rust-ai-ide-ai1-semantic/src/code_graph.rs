//! Code Graph Module
//! Builds and analyzes the relationship graph of code symbols across files and modules.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Code graph representing symbol relationships
#[derive(Debug, Clone)]
pub struct CodeGraph {
    pub nodes: HashMap<String, CodeNode>,
    pub edges: HashMap<String, RelationshipGraph>,
    pub files: HashSet<String>,
    pub modules: HashSet<String>,
}

/// Relationship graph for a specific symbol
#[derive(Debug, Clone)]
pub struct RelationshipGraph {
    pub symbol_id: String,
    pub relationships: Vec<CodeRelationship>,
    pub centrality: f32,
}

/// Code node representing a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeNode {
    pub id: String,
    pub name: String,
    pub kind: SymbolKind,
    pub location: CodeLocation,
    pub properties: HashMap<String, String>,
}

/// Type of code relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeRelationship {
    DependsOn,
    Calls,
    Implements,
    Inherits,
    References,
    Contains,
}

/// Local CodeLocation for the module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            files: HashSet::new(),
            modules: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, node: CodeNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_relationship(&mut self, from: &str, to: &str, relationship: CodeRelationship) {
        let from_key = from.to_string();
        if !self.edges.contains_key(&from_key) {
            self.edges.insert(from_key.clone(), RelationshipGraph {
                symbol_id: from_key.clone(),
                relationships: vec![],
                centrality: 0.0,
            });
        }

        if let Some(graph) = self.edges.get_mut(&from_key) {
            graph.relationships.push(relationship);
        }
    }

    pub fn calculate_centrality(&mut self) {
        // Simple centrality calculation
        for (id, node) in &self.nodes {
            let mut centrality = 0.0;
            if let Some(graph) = self.edges.get(id) {
                centrality = graph.relationships.len() as f32;
            }
            if let Some(graph) = self.edges.get_mut(id) {
                graph.centrality = centrality;
            }
        }
    }
}

impl RelationshipGraph {
    pub fn new(symbol_id: String) -> Self {
        Self {
            symbol_id,
            relationships: vec![],
            centrality: 0.0,
        }
    }
}

/// Symbol kind for code graph nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Variable,
    Constant,
    Macro,
}