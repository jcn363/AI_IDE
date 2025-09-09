//! SHARED TYPES - BACKWARD COMPATIBILITY REDIRECT
//!
//! IMPORTANT: This file now redirects to the consolidated shared types.
//! New code should import directly from '../shared/types' for better type safety.
//! This file is maintained only for existing imports to continue working.

// Re-export all types from the consolidated shared location
export * from '../shared/types/index';

// ===== COMPATIBILITY NOTE =====
// DEPRECATED: Use imports from '../shared/types/index' for new code.
// This file is maintained for backward compatibility only.
// Existing imports will continue to work but should eventually be migrated.
