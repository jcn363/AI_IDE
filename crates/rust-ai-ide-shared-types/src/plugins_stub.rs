//! Plugin system stub for when plugins feature is disabled
//!
//! This module provides stub implementations for plugin functionality
//! when the plugins feature is not enabled, preventing compilation errors.

use crate::errors::PluginError;
use crate::types::{GeneratedCode, ParsedType, TransformationContext};
use async_trait::async_trait;

/// Stub implementation of plugin system when plugins are disabled
#[derive(Debug)]
pub struct PluginSystem;

/// Stub implementation of type transformer plugin
#[derive(Debug)]
pub struct TypeTransformerPlugin;

/// Stub implementation that does nothing
impl PluginSystem {
    pub fn new() -> Result<Self, PluginError> {
        Ok(Self)
    }

    pub async fn load_plugins(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    pub async fn unload_plugins(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    pub fn get_transformer_plugins(&self) -> Vec<Box<dyn TransformerPluginTrait>> {
        vec![]
    }
}

/// Stub trait for transformer plugins
#[async_trait]
pub trait TransformerPluginTrait: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn transform_type(
        &self,
        _rust_type: &ParsedType,
        _context: &TransformationContext,
    ) -> Result<Option<GeneratedCode>, PluginError> {
        Ok(None)
    }
    async fn transform_field(
        &self,
        _field_name: &str,
        _field_type: &str,
        _context: &TransformationContext,
    ) -> Result<Option<String>, PluginError> {
        Ok(None)
    }
}

impl TypeTransformerPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransformerPluginTrait for TypeTransformerPlugin {
    fn name(&self) -> &str {
        "stub-transformer"
    }

    fn version(&self) -> &str {
        "0.0.0"
    }
}
