//! Bridge functions and API utilities
//!
//! This module contains functions that serve as bridges between different parts of the system.
//! These functions handle API response creation and other bridging functionality.

use crate::types::*;
use crate::error::*;

// Example usage functions (these would be in your actual application)

/// Example: Create a user response
pub fn create_user_response(user: User) -> ApiResponse<User> {
    ApiResponse {
        success:    true,
        data:       Some(user),
        error:      None,
        timestamp:  chrono::Utc::now().naive_utc(),
        request_id: uuid::Uuid::new_v4().to_string(),
    }
}

/// Example: Create an error response
pub fn create_error_response(code: &str, message: &str) -> ApiResponse<String> {
    ApiResponse {
        success:    false,
        data:       None,
        error:      Some(message.to_string()),
        timestamp:  chrono::Utc::now().naive_utc(),
        request_id: uuid::Uuid::new_v4().to_string(),
    }
}

/// Example: Create a paginated response
pub fn create_paginated_response<T>(items: Vec<T>, page: u32, per_page: u32, total: u64) -> PaginatedResponse<T> {
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    PaginatedResponse {
        data:       items,
        pagination: PaginationMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    }
}