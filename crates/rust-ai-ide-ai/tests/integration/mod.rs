//! # Integration Tests for Refactoring System
//!
//! This module contains comprehensive integration tests that verify the entire
//! refactoring system works correctly from end-to-end, including:
//!
//! - Backend command processing
//! - AI/LSP integration
//! - Error handling and recovery
//! - Batch operations
//! - Performance and reliability

pub mod refactoring_api_tests;
pub mod ai_lsp_integration_tests;
pub mod batch_operations_tests;
pub mod error_handling_tests;
pub mod performance_tests;