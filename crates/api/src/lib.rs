//! REST API server and endpoints.
//!
//! This crate provides a REST API for the CLMM LP Strategy Optimizer:
//! - Position management endpoints
//! - Strategy configuration and execution
//! - Pool information and analytics
//! - Real-time WebSocket updates
//! - OpenAPI documentation with Swagger UI
//! - JWT and API key authentication

/// Prelude module for convenient imports.
pub mod prelude;

/// Authentication module.
pub mod auth;
/// Error types.
pub mod error;
/// Request handlers.
pub mod handlers;
/// Middleware components.
pub mod middleware;
/// API request/response models.
pub mod models;
/// OpenAPI documentation.
pub mod openapi;
/// Route definitions.
pub mod routes;
/// Server configuration and startup.
pub mod server;
/// Service layer for API operations.
pub mod services;
/// Application state.
pub mod state;
/// WebSocket handlers.
pub mod websocket;

pub use auth::{AuthConfig, AuthError, AuthState, Claims, Role};
pub use error::ApiError;
pub use openapi::ApiDoc;
pub use server::{ApiServer, ServerConfig};
pub use services::{PositionService, StrategyService};
pub use state::AppState;
