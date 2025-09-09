//! Debugger caching for performance optimization
//!
//! This module provides caching for debugger state, breakpoints, and execution context
//! using the unified cache infrastructure from rust-ai-ide-cache.

use rust_ai_ide_cache::{key_utils, Cache, CacheConfig, CacheStats, InMemoryCache};
use rust_ai_ide_errors::IDEResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::types::{BreakpointInfo, DebuggerState, StackFrame, VariableInfo};

/// Cached breakpoint collection entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBreakpoints {
    /// File path breakpoints belong to
    pub file_path: PathBuf,
    /// Collection of breakpoints for this file
    pub breakpoints: Vec<BreakpointInfo>,
    /// Cache timestamp
    pub cached_at: SystemTime,
}

/// Cached debugger state entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDebuggerState {
    /// Session identifier (project/session ID)
    pub session_id: String,
    /// Current debugger state
    pub state: DebuggerState,
    /// Stack frames at current pause point
    pub stack_frames: Vec<StackFrame>,
    /// Variable values at current scope
    pub variables: HashMap<String, VariableInfo>,
    /// Cache timestamp
    pub cached_at: SystemTime,
}

/// Debugger cache for performance optimization
pub struct DebuggerCache {
    unified_cache: InMemoryCache<String, serde_json::Value>,
    config: CacheConfig,
}

impl std::fmt::Debug for DebuggerCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggerCache")
            .field("config", &self.config)
            .finish()
    }
}

impl DebuggerCache {
    /// Create a new debugger cache with optimized settings
    pub fn new() -> Self {
        let config = CacheConfig {
            max_entries: Some(500), // Debugger sessions don't need as much storage
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour for debugging state
            ..Default::default()
        };

        let unified_cache = InMemoryCache::new(&config);

        Self {
            unified_cache,
            config,
        }
    }

    /// Create cache with custom configuration
    pub fn new_with_config(config: CacheConfig) -> Self {
        let unified_cache = InMemoryCache::new(&config);
        Self {
            unified_cache,
            config,
        }
    }

    /// Cache breakpoints for a file
    pub async fn cache_breakpoints(
        &self,
        file_path: PathBuf,
        breakpoints: Vec<BreakpointInfo>,
    ) -> IDEResult<()> {
        let cache_key = key_utils::structured_key(
            "debugger_breakpoints",
            &[file_path.to_string_lossy().to_string().as_str()],
        );

        let cached = CachedBreakpoints {
            file_path,
            breakpoints,
            cached_at: SystemTime::now(),
        };

        let json_value = serde_json::to_value(&cached).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            )
        })?;

        self.unified_cache
            .insert(
                cache_key,
                json_value,
                Some(Duration::from_secs(1800)), // Shorter TTL for breakpoints
            )
            .await?;

        Ok(())
    }

    /// Get cached breakpoints for a file
    pub async fn get_breakpoints(&self, file_path: &PathBuf) -> Option<Vec<BreakpointInfo>> {
        let cache_key = key_utils::structured_key(
            "debugger_breakpoints",
            &[file_path.to_string_lossy().to_string().as_str()],
        );

        if let Ok(Some(value)) = self.unified_cache.get(&cache_key).await {
            match serde_json::from_value::<CachedBreakpoints>(value) {
                Ok(cached) => {
                    // Update access time by re-caching
                    let _ = self
                        .cache_breakpoints(cached.file_path.clone(), cached.breakpoints.clone())
                        .await;
                    Some(cached.breakpoints)
                }
                Err(_) => {
                    // Remove invalid cache entry
                    let _ = self.unified_cache.remove(&cache_key).await;
                    None
                }
            }
        } else {
            None
        }
    }

    /// Cache debugger state for a session
    pub async fn cache_debugger_state(
        &self,
        session_id: String,
        state: DebuggerState,
        stack_frames: Vec<StackFrame>,
        variables: HashMap<String, VariableInfo>,
    ) -> IDEResult<()> {
        let cache_key = key_utils::structured_key("debugger_state", &[&session_id]);

        let cached = CachedDebuggerState {
            session_id: session_id.clone(),
            state,
            stack_frames,
            variables,
            cached_at: SystemTime::now(),
        };

        let json_value = serde_json::to_value(&cached).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            )
        })?;

        self.unified_cache
            .insert(
                cache_key,
                json_value,
                Some(Duration::from_secs(900)), // 15 minutes for debugging state
            )
            .await?;

        Ok(())
    }

    /// Get cached debugger state for a session
    pub async fn get_debugger_state(&self, session_id: &str) -> Option<CachedDebuggerState> {
        let cache_key = key_utils::structured_key("debugger_state", &[session_id]);

        if let Ok(Some(value)) = self.unified_cache.get(&cache_key).await {
            match serde_json::from_value::<CachedDebuggerState>(value) {
                Ok(cached) => Some(cached),
                Err(_) => {
                    // Remove invalid cache entry
                    let _ = self.unified_cache.remove(&cache_key).await;
                    None
                }
            }
        } else {
            None
        }
    }

    /// Clear all debugger cache entries
    pub async fn clear(&self) -> IDEResult<()> {
        self.unified_cache.clear().await
    }

    /// Clear cache entries for a specific session
    pub async fn clear_session(&self, session_id: &str) -> IDEResult<()> {
        // Clear debugger state
        let state_key = key_utils::structured_key("debugger_state", &[session_id]);
        let _ = self.unified_cache.remove(&state_key).await;

        // Clear any breakpoints associated with the session
        // (In a real implementation, this would need a more sophisticated key mapping)
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.unified_cache.stats().await
    }

    /// Remove cached breakpoints for a file
    pub async fn invalidate_breakpoints(&self, file_path: &PathBuf) -> IDEResult<()> {
        let cache_key = key_utils::structured_key(
            "debugger_breakpoints",
            &[file_path.to_string_lossy().to_string().as_str()],
        );
        self.unified_cache.remove(&cache_key).await?;
        Ok(())
    }

    /// Get cache entry count
    pub async fn size(&self) -> usize {
        self.unified_cache.size().await
    }
}

impl Default for DebuggerCache {
    fn default() -> Self {
        Self::new()
    }
}
