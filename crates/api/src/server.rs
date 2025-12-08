//! Server configuration and startup.

use crate::handlers::health::init_start_time;
use crate::middleware::{RateLimiter, request_logging};
use crate::openapi::ApiDoc;
use crate::routes::create_versioned_router;
use crate::state::{ApiConfig, AppState};
use axum::{Router, middleware};
use clmm_lp_protocols::prelude::RpcConfig;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to.
    pub host: String,
    /// Port to bind to.
    pub port: u16,
    /// RPC configuration.
    pub rpc_config: RpcConfig,
    /// API configuration.
    pub api_config: ApiConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            rpc_config: RpcConfig::default(),
            api_config: ApiConfig::default(),
        }
    }
}

/// API server.
pub struct ApiServer {
    /// Server configuration.
    config: ServerConfig,
    /// Application state.
    state: AppState,
}

impl ApiServer {
    /// Creates a new API server.
    pub fn new(config: ServerConfig) -> Self {
        let state = AppState::new(config.rpc_config.clone(), config.api_config.clone());
        Self { config, state }
    }

    /// Creates a new API server with custom state.
    pub fn with_state(config: ServerConfig, state: AppState) -> Self {
        Self { config, state }
    }

    /// Gets the application state.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Builds the router with all middleware.
    pub fn build_router(&self) -> Router {
        let _api_keys: HashSet<String> = self.config.api_config.api_keys.iter().cloned().collect();
        let _rate_limiter = Arc::new(RateLimiter::new(
            self.config.api_config.rate_limit_per_minute,
        ));

        let mut router = create_versioned_router(self.state.clone());

        // Add Swagger UI at /docs
        router =
            router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));

        // Add middleware
        router = router.layer(middleware::from_fn(request_logging));

        // Add CORS if enabled
        if self.config.api_config.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            router = router.layer(cors);
        }

        // Add timeout
        #[allow(deprecated)]
        {
            router = router.layer(TimeoutLayer::new(Duration::from_secs(
                self.config.api_config.request_timeout_secs,
            )));
        }

        // Add tracing
        router = router.layer(TraceLayer::new_for_http());

        router
    }

    /// Starts the server.
    pub async fn run(self) -> anyhow::Result<()> {
        init_start_time();

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        let router = self.build_router();

        info!(address = %addr, "Starting API server");

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }

    /// Starts the server with graceful shutdown.
    pub async fn run_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> anyhow::Result<()> {
        init_start_time();

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        let router = self.build_router();

        info!(address = %addr, "Starting API server with graceful shutdown");

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        info!("API server stopped");

        Ok(())
    }
}

/// Creates a shutdown signal that listens for Ctrl+C.
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    info!("Shutdown signal received");
}
