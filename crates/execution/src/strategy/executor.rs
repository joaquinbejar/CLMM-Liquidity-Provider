//! Strategy executor for automated position management.

use super::{
    Decision, DecisionConfig, DecisionContext, DecisionEngine, RebalanceConfig, RebalanceExecutor,
    RebalanceParams,
};
use crate::emergency::CircuitBreaker;
use crate::lifecycle::{LifecycleTracker, RebalanceReason};
use crate::monitor::PositionMonitor;
use crate::transaction::TransactionManager;
use crate::wallet::Wallet;
use clmm_lp_protocols::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Configuration for strategy execution.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Evaluation interval in seconds.
    pub eval_interval_secs: u64,
    /// Whether to execute decisions automatically.
    pub auto_execute: bool,
    /// Whether to require confirmation before executing.
    pub require_confirmation: bool,
    /// Maximum slippage tolerance (as percentage).
    pub max_slippage_pct: Decimal,
    /// Dry run mode - simulate but don't execute.
    pub dry_run: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            eval_interval_secs: 300, // 5 minutes
            auto_execute: false,     // Require manual confirmation by default
            require_confirmation: true,
            max_slippage_pct: Decimal::new(5, 3), // 0.5%
            dry_run: false,
        }
    }
}

/// Strategy executor for automated position management.
pub struct StrategyExecutor {
    /// Position monitor.
    monitor: Arc<PositionMonitor>,
    /// Decision engine.
    decision_engine: DecisionEngine,
    /// Transaction manager.
    #[allow(dead_code)]
    tx_manager: Arc<TransactionManager>,
    /// Rebalance executor.
    rebalance_executor: RebalanceExecutor,
    /// Circuit breaker.
    circuit_breaker: Arc<CircuitBreaker>,
    /// Lifecycle tracker.
    lifecycle: Arc<LifecycleTracker>,
    /// Wallet for signing.
    wallet: Option<Arc<Wallet>>,
    /// Configuration.
    config: ExecutorConfig,
    /// Running flag.
    running: std::sync::atomic::AtomicBool,
    /// Pool reader for fetching state.
    pool_reader: WhirlpoolReader,
}

impl StrategyExecutor {
    /// Creates a new strategy executor.
    pub fn new(
        provider: Arc<RpcProvider>,
        monitor: Arc<PositionMonitor>,
        tx_manager: Arc<TransactionManager>,
        config: ExecutorConfig,
    ) -> Self {
        let lifecycle = Arc::new(LifecycleTracker::new());
        let circuit_breaker = Arc::new(CircuitBreaker::default());
        let pool_reader = WhirlpoolReader::new(provider.clone());

        let mut rebalance_executor = RebalanceExecutor::new(
            provider,
            tx_manager.clone(),
            lifecycle.clone(),
            RebalanceConfig::default(),
        );
        rebalance_executor.set_dry_run(config.dry_run);

        Self {
            monitor,
            decision_engine: DecisionEngine::default(),
            tx_manager,
            rebalance_executor,
            circuit_breaker,
            lifecycle,
            wallet: None,
            config,
            running: std::sync::atomic::AtomicBool::new(false),
            pool_reader,
        }
    }

    /// Sets the wallet for signing transactions.
    pub fn set_wallet(&mut self, wallet: Arc<Wallet>) {
        self.wallet = Some(wallet.clone());
        self.rebalance_executor.set_wallet(wallet);
    }

    /// Sets the decision engine configuration.
    pub fn set_decision_config(&mut self, config: DecisionConfig) {
        self.decision_engine.set_config(config);
    }

    /// Enables or disables dry run mode.
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.config.dry_run = dry_run;
        self.rebalance_executor.set_dry_run(dry_run);
    }

    /// Gets the circuit breaker.
    pub fn circuit_breaker(&self) -> &Arc<CircuitBreaker> {
        &self.circuit_breaker
    }

    /// Gets the lifecycle tracker.
    pub fn lifecycle(&self) -> &Arc<LifecycleTracker> {
        &self.lifecycle
    }

    /// Starts the strategy execution loop.
    pub async fn start(&self) {
        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let eval_interval = Duration::from_secs(self.config.eval_interval_secs);
        let mut ticker = interval(eval_interval);

        info!(
            interval_secs = self.config.eval_interval_secs,
            auto_execute = self.config.auto_execute,
            dry_run = self.config.dry_run,
            "Starting strategy executor"
        );

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            ticker.tick().await;

            // Check circuit breaker
            if !self.circuit_breaker.is_allowed().await {
                warn!("Circuit breaker open, skipping evaluation");
                continue;
            }

            if let Err(e) = self.evaluate_all().await {
                error!(error = %e, "Strategy evaluation failed");
                self.circuit_breaker.record_failure().await;
            } else {
                self.circuit_breaker.record_success().await;
            }
        }

        info!("Strategy executor stopped");
    }

    /// Stops the strategy execution loop.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Evaluates all monitored positions.
    async fn evaluate_all(&self) -> anyhow::Result<()> {
        let positions = self.monitor.get_positions().await;

        debug!(count = positions.len(), "Evaluating positions");

        for position in positions {
            if let Err(e) = self.evaluate_position(&position).await {
                warn!(
                    position = %position.address,
                    error = %e,
                    "Failed to evaluate position"
                );
            }
        }

        Ok(())
    }

    /// Evaluates a single position.
    async fn evaluate_position(
        &self,
        position: &crate::monitor::MonitoredPosition,
    ) -> anyhow::Result<()> {
        // Fetch current pool state
        let pool = self
            .pool_reader
            .get_pool_state(&position.pool.to_string())
            .await
            .unwrap_or_else(|_| WhirlpoolState {
                address: position.pool.to_string(),
                token_mint_a: solana_sdk::pubkey::Pubkey::default(),
                token_mint_b: solana_sdk::pubkey::Pubkey::default(),
                tick_current: 0,
                tick_spacing: 64,
                sqrt_price: 1 << 64,
                price: Decimal::ONE,
                liquidity: 0,
                fee_rate_bps: 30,
                protocol_fee_rate_bps: 0,
                fee_growth_global_a: 0,
                fee_growth_global_b: 0,
            });

        // Calculate hours since last rebalance from lifecycle
        let hours_since_rebalance = self
            .calculate_hours_since_rebalance(&position.address)
            .await;

        let context = DecisionContext {
            position: position.clone(),
            pool: pool.clone(),
            hours_since_rebalance,
        };

        let decision = self.decision_engine.decide(&context);

        if decision.requires_transaction() {
            info!(
                position = %position.address,
                decision = %decision.description(),
                dry_run = self.config.dry_run,
                "Decision requires action"
            );

            if self.config.auto_execute {
                self.execute_decision(position, &decision, &pool).await?;
            }
        }

        Ok(())
    }

    /// Calculates hours since last rebalance.
    async fn calculate_hours_since_rebalance(&self, position: &solana_sdk::pubkey::Pubkey) -> u64 {
        let events = self.lifecycle.get_events(position).await;

        // Find the last rebalance event
        for event in events.iter().rev() {
            if event.event_type == crate::lifecycle::LifecycleEventType::Rebalanced {
                let duration = chrono::Utc::now() - event.timestamp;
                return duration.num_hours().max(0) as u64;
            }
        }

        // If no rebalance, use position open time
        if let Some(summary) = self.lifecycle.get_summary(position).await {
            let duration = chrono::Utc::now() - summary.opened_at;
            return duration.num_hours().max(0) as u64;
        }

        // Default to a large value to allow rebalancing
        u64::MAX
    }

    /// Executes a decision.
    async fn execute_decision(
        &self,
        position: &crate::monitor::MonitoredPosition,
        decision: &Decision,
        _pool: &WhirlpoolState,
    ) -> anyhow::Result<()> {
        info!(
            position = %position.address,
            decision = %decision.description(),
            "Executing decision"
        );

        match decision {
            Decision::Hold => {
                // Nothing to do
            }
            Decision::Rebalance {
                new_tick_lower,
                new_tick_upper,
            } => {
                let params = RebalanceParams {
                    position: position.address,
                    pool: position.pool,
                    current_tick_lower: position.on_chain.tick_lower,
                    current_tick_upper: position.on_chain.tick_upper,
                    new_tick_lower: *new_tick_lower,
                    new_tick_upper: *new_tick_upper,
                    current_liquidity: position.on_chain.liquidity,
                    reason: if !position.in_range {
                        RebalanceReason::RangeExit
                    } else {
                        RebalanceReason::ILThreshold
                    },
                    current_il_pct: position.pnl.il_pct,
                };

                let result = self.rebalance_executor.execute(params).await;

                if !result.success
                    && let Some(err) = result.error
                {
                    error!(error = %err, "Rebalance failed");
                }
            }
            Decision::Close => {
                info!("Would execute close");
                // TODO: Implement close via emergency exit manager
            }
            Decision::IncreaseLiquidity { amount } => {
                info!(amount = %amount, "Would execute increase liquidity");
            }
            Decision::DecreaseLiquidity { amount } => {
                info!(amount = %amount, "Would execute decrease liquidity");
            }
            Decision::CollectFees => {
                info!("Would execute collect fees");
            }
        }

        Ok(())
    }
}
