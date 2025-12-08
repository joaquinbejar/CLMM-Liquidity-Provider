//! OpenAPI documentation configuration.
//!
//! Provides Swagger UI and OpenAPI spec generation using utoipa.

use crate::handlers;
use crate::models::{
    CreateStrategyRequest, HealthResponse, ListPoolsResponse, ListPositionsResponse,
    ListStrategiesResponse, MessageResponse, MetricsResponse, OpenPositionRequest, PnLResponse,
    PoolResponse, PoolStateResponse, PortfolioAnalyticsResponse, PositionResponse,
    RebalanceRequest, SimulationRequest, SimulationResponse, StrategyPerformanceResponse,
    StrategyResponse,
};
use utoipa::OpenApi;

/// OpenAPI documentation structure.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "CLMM LP Strategy Optimizer API",
        version = "0.1.1-alpha.2",
        description = "REST API for the CLMM Liquidity Provider Strategy Optimizer. \
                       Provides endpoints for position management, strategy automation, \
                       pool analysis, and portfolio analytics.",
        license(
            name = "MIT OR Apache-2.0",
            url = "https://github.com/joaquinbejar/CLMM-Liquidity-Provider"
        ),
        contact(
            name = "Joaquín Béjar García",
            email = "jb@taunais.com"
        )
    ),
    servers(
        (url = "/api/v1", description = "API v1")
    ),
    tags(
        (name = "Health", description = "Health check and metrics endpoints"),
        (name = "Positions", description = "LP position management"),
        (name = "Strategies", description = "Automated strategy management"),
        (name = "Pools", description = "Pool information and state"),
        (name = "Analytics", description = "Portfolio analytics and simulations")
    ),
    paths(
        // Health endpoints
        handlers::health_check,
        handlers::liveness,
        handlers::readiness,
        handlers::metrics,
        // Position endpoints
        handlers::list_positions,
        handlers::get_position,
        handlers::open_position,
        handlers::close_position,
        handlers::collect_fees,
        handlers::rebalance_position,
        handlers::get_position_pnl,
        // Strategy endpoints
        handlers::list_strategies,
        handlers::get_strategy,
        handlers::create_strategy,
        handlers::update_strategy,
        handlers::delete_strategy,
        handlers::start_strategy,
        handlers::stop_strategy,
        handlers::get_strategy_performance,
        // Pool endpoints
        handlers::list_pools,
        handlers::get_pool,
        handlers::get_pool_state,
        // Analytics endpoints
        handlers::get_portfolio_analytics,
        handlers::run_simulation,
    ),
    components(
        schemas(
            // Health
            HealthResponse,
            MetricsResponse,
            // Positions
            ListPositionsResponse,
            PositionResponse,
            PnLResponse,
            OpenPositionRequest,
            RebalanceRequest,
            MessageResponse,
            // Strategies
            ListStrategiesResponse,
            StrategyResponse,
            StrategyPerformanceResponse,
            CreateStrategyRequest,
            // Pools
            ListPoolsResponse,
            PoolResponse,
            PoolStateResponse,
            // Analytics
            PortfolioAnalyticsResponse,
            SimulationRequest,
            SimulationResponse,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security addon for OpenAPI.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Returns the OpenAPI JSON specification.
#[must_use]
pub fn openapi_json() -> String {
    ApiDoc::openapi().to_json().unwrap_or_default()
}

/// Returns the OpenAPI YAML specification.
#[must_use]
pub fn openapi_yaml() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let json = openapi_json();
        assert!(!json.is_empty());
        assert!(json.contains("CLMM LP Strategy Optimizer API"));
    }

    #[test]
    fn test_openapi_yaml() {
        let yaml = openapi_yaml();
        assert!(!yaml.is_empty());
        assert!(yaml.contains("CLMM LP Strategy Optimizer API"));
    }
}
