//! Strategy service for managing automated strategies.

use crate::error::ApiError;
use crate::state::{AlertUpdate, AppState};
use clmm_lp_execution::prelude::{DecisionConfig, ExecutorConfig, StrategyExecutor};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Result of a strategy operation.
#[derive(Debug, Clone)]
pub struct StrategyOperationResult {
    /// Whether the operation was successful.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

impl StrategyOperationResult {
    /// Creates a successful result.
    pub fn success() -> Self {
        Self {
            success: true,
            error: None,
        }
    }

    /// Creates a failed result.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Service for strategy operations.
pub struct StrategyService {
    /// Application state.
    state: AppState,
    /// Active strategy executors.
    executors: Arc<RwLock<std::collections::HashMap<String, Arc<RwLock<StrategyExecutor>>>>>,
}

impl StrategyService {
    /// Creates a new strategy service.
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            executors: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Starts a strategy.
    pub async fn start_strategy(
        &self,
        strategy_id: &str,
    ) -> Result<StrategyOperationResult, ApiError> {
        info!(strategy_id = %strategy_id, "Starting strategy");

        // Get strategy configuration
        let mut strategies = self.state.strategies.write().await;
        let strategy = strategies
            .get_mut(strategy_id)
            .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

        if strategy.running {
            return Err(ApiError::Conflict(
                "Strategy is already running".to_string(),
            ));
        }

        // Parse configuration
        let dry_run = strategy
            .config
            .get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let auto_execute = strategy
            .config
            .get("auto_execute")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let eval_interval_secs = strategy
            .config
            .get("parameters")
            .and_then(|p| p.get("eval_interval_secs"))
            .and_then(|v| v.as_u64())
            .unwrap_or(300);

        // Create executor configuration
        let executor_config = ExecutorConfig {
            eval_interval_secs,
            auto_execute,
            require_confirmation: !auto_execute,
            max_slippage_pct: Decimal::new(5, 3), // 0.5%
            dry_run,
        };

        // Create strategy executor
        let mut executor = StrategyExecutor::new(
            self.state.provider.clone(),
            self.state.monitor.clone(),
            self.state.tx_manager.clone(),
            executor_config,
        );

        // Configure decision engine if parameters provided
        if let Some(params) = strategy.config.get("parameters") {
            let mut decision_config = DecisionConfig::default();

            if let Some(threshold) = params.get("rebalance_threshold_pct")
                && let Some(val) = threshold.as_f64()
            {
                decision_config.il_rebalance_threshold =
                    Decimal::from_f64_retain(val / 100.0).unwrap_or(Decimal::new(5, 2));
            }

            if let Some(max_il) = params.get("max_il_pct")
                && let Some(val) = max_il.as_f64()
            {
                decision_config.il_close_threshold =
                    Decimal::from_f64_retain(val / 100.0).unwrap_or(Decimal::new(15, 2));
            }

            if let Some(min_hours) = params.get("min_rebalance_interval_hours")
                && let Some(val) = min_hours.as_u64()
            {
                decision_config.min_rebalance_interval_hours = val;
            }

            executor.set_decision_config(decision_config);
        }

        let executor = Arc::new(RwLock::new(executor));

        // Store executor
        {
            let mut executors = self.executors.write().await;
            executors.insert(strategy_id.to_string(), executor.clone());
        }

        // Start executor in background task
        let executor_clone = executor.clone();
        let strategy_id_clone = strategy_id.to_string();
        let alert_sender = self.state.alert_updates.clone();

        tokio::spawn(async move {
            info!(strategy_id = %strategy_id_clone, "Strategy executor task started");

            let executor_guard = executor_clone.read().await;
            executor_guard.start().await;

            // Notify when stopped
            let _ = alert_sender.send(AlertUpdate {
                level: "info".to_string(),
                message: format!("Strategy {} stopped", strategy_id_clone),
                timestamp: chrono::Utc::now(),
                position_address: None,
            });
        });

        // Update strategy state
        strategy.running = true;
        strategy.updated_at = chrono::Utc::now();

        // Broadcast alert
        self.state.broadcast_alert(AlertUpdate {
            level: "info".to_string(),
            message: format!("Strategy {} started", strategy_id),
            timestamp: chrono::Utc::now(),
            position_address: None,
        });

        info!(strategy_id = %strategy_id, "Strategy started successfully");
        Ok(StrategyOperationResult::success())
    }

    /// Stops a strategy.
    pub async fn stop_strategy(
        &self,
        strategy_id: &str,
    ) -> Result<StrategyOperationResult, ApiError> {
        info!(strategy_id = %strategy_id, "Stopping strategy");

        // Get strategy
        let mut strategies = self.state.strategies.write().await;
        let strategy = strategies
            .get_mut(strategy_id)
            .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

        if !strategy.running {
            return Err(ApiError::Conflict("Strategy is not running".to_string()));
        }

        // Stop executor
        {
            let executors = self.executors.read().await;
            if let Some(executor) = executors.get(strategy_id) {
                let executor_guard = executor.read().await;
                executor_guard.stop();
            }
        }

        // Remove executor
        {
            let mut executors = self.executors.write().await;
            executors.remove(strategy_id);
        }

        // Update strategy state
        strategy.running = false;
        strategy.updated_at = chrono::Utc::now();

        // Broadcast alert
        self.state.broadcast_alert(AlertUpdate {
            level: "info".to_string(),
            message: format!("Strategy {} stopped", strategy_id),
            timestamp: chrono::Utc::now(),
            position_address: None,
        });

        info!(strategy_id = %strategy_id, "Strategy stopped successfully");
        Ok(StrategyOperationResult::success())
    }

    /// Gets the executor for a strategy.
    pub async fn get_executor(&self, strategy_id: &str) -> Option<Arc<RwLock<StrategyExecutor>>> {
        let executors = self.executors.read().await;
        executors.get(strategy_id).cloned()
    }

    /// Triggers a manual evaluation for a strategy.
    pub async fn trigger_evaluation(
        &self,
        strategy_id: &str,
    ) -> Result<StrategyOperationResult, ApiError> {
        info!(strategy_id = %strategy_id, "Triggering manual evaluation");

        let executors = self.executors.read().await;
        let _executor = executors.get(strategy_id).ok_or_else(|| {
            ApiError::not_found("Strategy executor not found - is the strategy running?")
        })?;

        // The executor runs on its own schedule, but we can trigger by checking positions
        // For now, just verify it's running
        let strategies = self.state.strategies.read().await;
        let strategy = strategies
            .get(strategy_id)
            .ok_or_else(|| ApiError::not_found("Strategy not found"))?;

        if !strategy.running {
            return Err(ApiError::Conflict("Strategy is not running".to_string()));
        }

        info!(strategy_id = %strategy_id, "Evaluation will occur on next interval");
        Ok(StrategyOperationResult::success())
    }

    /// Gets statistics for a running strategy.
    pub async fn get_strategy_stats(
        &self,
        strategy_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let executors = self.executors.read().await;

        if let Some(executor) = executors.get(strategy_id) {
            let executor_guard = executor.read().await;
            let lifecycle = executor_guard.lifecycle();
            let circuit_breaker = executor_guard.circuit_breaker();

            let stats = lifecycle.get_aggregate_stats().await;
            let cb_stats = circuit_breaker.stats().await;
            let cb_state = circuit_breaker.state().await;

            Ok(serde_json::json!({
                "lifecycle": {
                    "total_positions": stats.total_positions,
                    "open_positions": stats.open_positions,
                    "closed_positions": stats.closed_positions,
                    "total_rebalances": stats.total_rebalances,
                    "total_fees_usd": stats.total_fees_usd.to_string(),
                    "total_pnl_usd": stats.total_pnl_usd.to_string(),
                    "avg_pnl_pct": stats.avg_pnl_pct.to_string(),
                    "total_tx_costs_lamports": stats.total_tx_costs_lamports
                },
                "circuit_breaker": {
                    "state": format!("{:?}", cb_state),
                    "success_count": cb_stats.success_count,
                    "failure_count": cb_stats.failure_count,
                    "manually_tripped": cb_stats.manually_tripped,
                    "opened_at": cb_stats.opened_at.map(|t| format!("{:?}", t))
                }
            }))
        } else {
            // Strategy not running, return basic stats from lifecycle
            let stats = self.state.lifecycle.get_aggregate_stats().await;

            Ok(serde_json::json!({
                "lifecycle": {
                    "total_positions": stats.total_positions,
                    "open_positions": stats.open_positions,
                    "closed_positions": stats.closed_positions,
                    "total_rebalances": stats.total_rebalances,
                    "total_fees_usd": stats.total_fees_usd.to_string(),
                    "total_pnl_usd": stats.total_pnl_usd.to_string(),
                    "avg_pnl_pct": stats.avg_pnl_pct.to_string(),
                    "total_tx_costs_lamports": stats.total_tx_costs_lamports
                },
                "circuit_breaker": null
            }))
        }
    }
}
