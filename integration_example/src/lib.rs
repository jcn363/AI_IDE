//! Integration Example: Shared Types Crate
//!
//! This example shows how to integrate the shared-types crate into your existing Rust project.
//! The types defined here will be automatically generated for multiple platforms.

use serde::{Deserialize, Serialize};

/// API Response wrapper
/// This type will be generated for TypeScript, Python, Go, GraphQL, and OpenAPI
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    /// Whether the request succeeded
    pub success: bool,

    /// Response data payload
    pub data: Option<T>,

    /// Error message if any
    pub error: Option<String>,

    /// Request timestamp
    pub timestamp: chrono::NaiveDateTime,

    /// Request ID for tracing
    pub request_id: String,
}

/// User entity - core domain model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// Unique user identifier (UUID as string)
    pub id: String,

    /// User's profile information
    pub profile: UserProfile,

    /// Account settings and preferences
    pub settings: UserSettings,

    /// User permissions
    pub permissions: Vec<String>,

    /// Account status
    pub status: AccountStatus,

    /// Registration timestamp
    pub created_at: chrono::NaiveDateTime,

    /// Last activity timestamp
    pub last_active: Option<chrono::NaiveDateTime>,
}

/// User profile information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserProfile {
    /// Display name
    pub display_name: String,

    /// Email address (optional)
    pub email: Option<String>,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// User's location
    pub location: String,

    /// User biography
    pub bio: Option<String>,

    /// Website URL
    pub website: Option<String>,
}

/// User account settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSettings {
    /// UI theme preference
    pub theme: Theme,

    /// Notification preferences
    pub notifications: NotificationSettings,

    /// User interface language
    pub language: Language,

    /// Privacy settings
    pub privacy: PrivacySettings,
}

/// Available UI themes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Theme {
    Light,
    Dark,
    System,
    Custom(String),
}

/// Notification preference settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationSettings {
    /// Email notifications enabled
    pub email: bool,

    /// Push notifications enabled
    pub push: bool,

    /// Desktop notifications enabled
    pub desktop: bool,

    /// Marketing emails
    pub marketing: bool,
}

/// Supported languages
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Language {
    En,
    Es,
    Fr,
    De,
    Ja,
    Zh,
    Ko,
}

/// User privacy settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivacySettings {
    /// Profile visibility
    pub profile_visibility: Visibility,

    /// Activity visibility
    pub activity_visibility: Visibility,

    /// Allow data collection for analytics
    pub allow_analytics: bool,
}

/// Visibility levels
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Visibility {
    Public,
    FriendsOnly,
    Private,
}

/// Account status
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AccountStatus {
    Active,
    Suspended,
    Deactivated,
    PendingVerification,
}

/// Product catalog item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    /// Product ID
    pub id: String,

    /// Product name
    pub name: String,

    /// Product description
    pub description: Option<String>,

    /// Price in cents
    pub price: i64,

    /// Product category
    pub category: String,

    /// Product tags
    pub tags: Vec<String>,

    /// Whether the product is active
    pub is_active: bool,

    /// Creation timestamp
    pub created_at: chrono::NaiveDateTime,

    /// Last updated timestamp
    pub updated_at: Option<chrono::NaiveDateTime>,
}

/// Order entity
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    /// Order ID
    pub id: String,

    /// Customer ID
    pub customer_id: String,

    /// Order line items
    pub items: Vec<OrderItem>,

    /// Order status
    pub status: OrderStatus,

    /// Total amount in cents
    pub total: i64,

    /// Order creation timestamp
    pub created_at: chrono::NaiveDateTime,

    /// Order fulfillment timestamp
    pub fulfilled_at: Option<chrono::NaiveDateTime>,
}

/// Order line item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItem {
    /// Product ID
    pub product_id: String,

    /// Quantity ordered
    pub quantity: u32,

    /// Unit price in cents
    pub unit_price: i64,

    /// Optional discount percentage
    pub discount: Option<f64>,
}

/// Order status
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

/// Pagination metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginationMeta {
    /// Current page (1-based)
    pub page: u32,

    /// Number of items per page
    pub per_page: u32,

    /// Total number of items
    pub total: u64,

    /// Total number of pages
    pub total_pages: u32,
}

/// Paginated API response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginatedResponse<T> {
    /// Response data
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Error response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Additional error details
    pub details: Option<serde_json::Value>,
}

// Example usage functions (these would be in your actual application)

/// Example: Create a user response
pub fn create_user_response(user: User) -> ApiResponse<User> {
    ApiResponse {
        success: true,
        data: Some(user),
        error: None,
        timestamp: chrono::Utc::now().naive_utc(),
        request_id: uuid::Uuid::new_v4().to_string(),
    }
}

/// Example: Create an error response
pub fn create_error_response(code: &str, message: &str) -> ApiResponse<String> {
    ApiResponse {
        success: false,
        data: None,
        error: Some(message.to_string()),
        timestamp: chrono::Utc::now().naive_utc(),
        request_id: uuid::Uuid::new_v4().to_string(),
    }
}

/// Example: Create a paginated response
pub fn create_paginated_response<T>(
    items: Vec<T>,
    page: u32,
    per_page: u32,
    total: u64,
) -> PaginatedResponse<T> {
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    PaginatedResponse {
        data: items,
        pagination: PaginationMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    }
}