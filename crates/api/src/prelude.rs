//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_api::prelude::*;
//! ```

// Error types
pub use crate::error::{ApiError, ApiResult, ErrorResponse};

// Models
pub use crate::models::{
    CircuitBreakerStatus, ComponentHealth, CreateStrategyRequest, HealthResponse,
    ListPoolsResponse, ListPositionsResponse, ListStrategiesResponse, MessageResponse,
    MetricsResponse, OpenPositionRequest, PnLResponse, PoolResponse, PoolStateResponse,
    PortfolioAnalyticsResponse, PositionResponse, PositionStatus, RebalanceRequest, ServiceStatus,
    SimulationRequest, SimulationResponse, StrategyParameters, StrategyPerformanceResponse,
    StrategyResponse, StrategyType, SuccessResponse,
};

// Server
pub use crate::server::{ApiServer, ServerConfig, shutdown_signal};

// State
pub use crate::state::{AlertUpdate, ApiConfig, AppState, PositionUpdate, StrategyState};

// Middleware
pub use crate::middleware::RateLimiter;

// Routes
pub use crate::routes::{create_router, create_versioned_router};

// Services
pub use crate::services::{PositionService, StrategyService};

// Authentication
pub use crate::auth::{AuthConfig, AuthError, AuthState, Claims, Role};

// OpenAPI
pub use crate::openapi::ApiDoc;
