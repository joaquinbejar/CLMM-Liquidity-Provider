//! Rebalancing execution logic.

use crate::lifecycle::{FeesCollectedData, LifecycleTracker, RebalanceData, RebalanceReason};
use crate::transaction::TransactionManager;
use crate::wallet::Wallet;
use clmm_lp_protocols::prelude::*;
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Configuration for rebalancing.
#[derive(Debug, Clone)]
pub struct RebalanceConfig {
    /// Maximum slippage tolerance in basis points.
    pub max_slippage_bps: u16,
    /// Minimum profit multiplier for rebalance to be worthwhile.
    pub min_profit_multiplier: Decimal,
    /// Whether to collect fees before rebalancing.
    pub collect_fees_first: bool,
    /// Priority fee level.
    pub priority_level: crate::transaction::PriorityLevel,
}

impl Default for RebalanceConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 50,                      // 0.5%
            min_profit_multiplier: Decimal::new(2, 0), // 2x tx cost
            collect_fees_first: true,
            priority_level: crate::transaction::PriorityLevel::Medium,
        }
    }
}

/// Parameters for a rebalance operation.
#[derive(Debug, Clone)]
pub struct RebalanceParams {
    /// Position to rebalance.
    pub position: Pubkey,
    /// Pool address.
    pub pool: Pubkey,
    /// Current tick lower.
    pub current_tick_lower: i32,
    /// Current tick upper.
    pub current_tick_upper: i32,
    /// New tick lower.
    pub new_tick_lower: i32,
    /// New tick upper.
    pub new_tick_upper: i32,
    /// Current liquidity.
    pub current_liquidity: u128,
    /// Reason for rebalancing.
    pub reason: RebalanceReason,
    /// Current IL percentage.
    pub current_il_pct: Decimal,
}

/// Result of a rebalance operation.
#[derive(Debug, Clone)]
pub struct RebalanceResult {
    /// Whether rebalance was successful.
    pub success: bool,
    /// Old position address.
    pub old_position: Pubkey,
    /// New position address (if created).
    pub new_position: Option<Pubkey>,
    /// Fees collected.
    pub fees_collected: Option<(u64, u64)>,
    /// Liquidity removed from old position.
    pub liquidity_removed: u128,
    /// Liquidity added to new position.
    pub liquidity_added: u128,
    /// Transaction cost in lamports.
    pub tx_cost_lamports: u64,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Executor for rebalancing operations.
pub struct RebalanceExecutor {
    /// RPC provider.
    #[allow(dead_code)]
    provider: Arc<RpcProvider>,
    /// Transaction manager.
    #[allow(dead_code)]
    tx_manager: Arc<TransactionManager>,
    /// Wallet for signing.
    wallet: Option<Arc<Wallet>>,
    /// Lifecycle tracker.
    lifecycle: Arc<LifecycleTracker>,
    /// Configuration.
    config: RebalanceConfig,
    /// Dry run mode.
    dry_run: bool,
}

impl RebalanceExecutor {
    /// Creates a new rebalance executor.
    pub fn new(
        provider: Arc<RpcProvider>,
        tx_manager: Arc<TransactionManager>,
        lifecycle: Arc<LifecycleTracker>,
        config: RebalanceConfig,
    ) -> Self {
        Self {
            provider,
            tx_manager,
            wallet: None,
            lifecycle,
            config,
            dry_run: false,
        }
    }

    /// Sets the wallet for signing.
    pub fn set_wallet(&mut self, wallet: Arc<Wallet>) {
        self.wallet = Some(wallet);
    }

    /// Enables or disables dry run mode.
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Checks if a rebalance is profitable.
    pub async fn is_profitable(&self, params: &RebalanceParams) -> ProfitabilityCheck {
        // Estimate transaction costs
        let estimated_tx_cost = self.estimate_tx_cost().await;

        // Estimate expected benefit from rebalancing
        let expected_benefit = self.estimate_benefit(params).await;

        let is_profitable =
            expected_benefit > Decimal::from(estimated_tx_cost) * self.config.min_profit_multiplier;

        ProfitabilityCheck {
            is_profitable,
            estimated_tx_cost,
            expected_benefit,
            min_required_benefit: Decimal::from(estimated_tx_cost)
                * self.config.min_profit_multiplier,
        }
    }

    /// Estimates transaction cost for rebalancing.
    async fn estimate_tx_cost(&self) -> u64 {
        // Base cost: ~5000 lamports per signature + compute units
        // Rebalance involves: collect fees + decrease liquidity + close position + open position + increase liquidity
        // Estimate ~0.01 SOL total
        10_000_000 // 0.01 SOL in lamports
    }

    /// Estimates expected benefit from rebalancing.
    async fn estimate_benefit(&self, params: &RebalanceParams) -> Decimal {
        // Simplified estimation based on IL recovery
        // In a real implementation, this would use historical data and simulations
        let il_recovery = params.current_il_pct.abs() * Decimal::new(5, 1); // Assume 50% IL recovery
        il_recovery * Decimal::from(1000) // Convert to USD equivalent
    }

    /// Executes a rebalance operation.
    pub async fn execute(&self, params: RebalanceParams) -> RebalanceResult {
        info!(
            position = %params.position,
            old_range = format!("[{}, {}]", params.current_tick_lower, params.current_tick_upper),
            new_range = format!("[{}, {}]", params.new_tick_lower, params.new_tick_upper),
            reason = ?params.reason,
            dry_run = self.dry_run,
            "Executing rebalance"
        );

        let mut result = RebalanceResult {
            success: false,
            old_position: params.position,
            new_position: None,
            fees_collected: None,
            liquidity_removed: 0,
            liquidity_added: 0,
            tx_cost_lamports: 0,
            error: None,
        };

        // Check profitability
        let profitability = self.is_profitable(&params).await;
        if !profitability.is_profitable {
            warn!(
                expected_benefit = %profitability.expected_benefit,
                min_required = %profitability.min_required_benefit,
                "Rebalance not profitable, skipping"
            );
            result.error = Some("Rebalance not profitable".to_string());
            return result;
        }

        if self.dry_run {
            info!("Dry run mode - simulating rebalance");
            result.success = true;
            result.liquidity_removed = params.current_liquidity;
            result.liquidity_added = params.current_liquidity;
            return result;
        }

        // Step 1: Collect fees if configured
        if self.config.collect_fees_first {
            match self.collect_fees(&params.position).await {
                Ok(fees) => {
                    result.fees_collected = Some(fees);
                    result.tx_cost_lamports += 5000; // Approximate

                    // Record in lifecycle
                    self.lifecycle
                        .record_fees_collected(
                            params.position,
                            params.pool,
                            FeesCollectedData {
                                fees_a: fees.0,
                                fees_b: fees.1,
                                fees_usd: Decimal::ZERO, // Would need price oracle
                            },
                        )
                        .await;
                }
                Err(e) => {
                    warn!(error = %e, "Failed to collect fees, continuing");
                }
            }
        }

        // Step 2: Decrease liquidity from current position
        match self
            .decrease_liquidity(&params.position, params.current_liquidity)
            .await
        {
            Ok(liquidity) => {
                result.liquidity_removed = liquidity;
                result.tx_cost_lamports += 5000;
            }
            Err(e) => {
                error!(error = %e, "Failed to decrease liquidity");
                result.error = Some(e.to_string());
                return result;
            }
        }

        // Step 3: Close old position
        if let Err(e) = self.close_position(&params.position).await {
            error!(error = %e, "Failed to close position");
            result.error = Some(e.to_string());
            return result;
        }
        result.tx_cost_lamports += 5000;

        // Step 4: Open new position
        let new_position = match self
            .open_position(&params.pool, params.new_tick_lower, params.new_tick_upper)
            .await
        {
            Ok(pos) => pos,
            Err(e) => {
                error!(error = %e, "Failed to open new position");
                result.error = Some(e.to_string());
                return result;
            }
        };
        result.new_position = Some(new_position);
        result.tx_cost_lamports += 5000;

        // Step 5: Increase liquidity in new position
        match self
            .increase_liquidity(&new_position, params.current_liquidity)
            .await
        {
            Ok(liquidity) => {
                result.liquidity_added = liquidity;
                result.tx_cost_lamports += 5000;
            }
            Err(e) => {
                error!(error = %e, "Failed to increase liquidity");
                result.error = Some(e.to_string());
                return result;
            }
        }

        // Record rebalance in lifecycle
        self.lifecycle
            .record_rebalance(
                new_position,
                params.pool,
                RebalanceData {
                    old_tick_lower: params.current_tick_lower,
                    old_tick_upper: params.current_tick_upper,
                    new_tick_lower: params.new_tick_lower,
                    new_tick_upper: params.new_tick_upper,
                    old_liquidity: params.current_liquidity,
                    new_liquidity: result.liquidity_added,
                    tx_cost_lamports: result.tx_cost_lamports,
                    il_at_rebalance: params.current_il_pct,
                    reason: params.reason,
                },
            )
            .await;

        result.success = true;
        info!(
            old_position = %params.position,
            new_position = %new_position,
            tx_cost = result.tx_cost_lamports,
            "Rebalance completed successfully"
        );

        result
    }

    /// Collects fees from a position.
    async fn collect_fees(&self, _position: &Pubkey) -> anyhow::Result<(u64, u64)> {
        // TODO: Implement actual fee collection via Whirlpool instruction
        debug!("Would collect fees");
        Ok((0, 0))
    }

    /// Decreases liquidity from a position.
    async fn decrease_liquidity(
        &self,
        _position: &Pubkey,
        liquidity: u128,
    ) -> anyhow::Result<u128> {
        // TODO: Implement actual liquidity decrease via Whirlpool instruction
        debug!(liquidity = liquidity, "Would decrease liquidity");
        Ok(liquidity)
    }

    /// Closes a position.
    async fn close_position(&self, _position: &Pubkey) -> anyhow::Result<()> {
        // TODO: Implement actual position close via Whirlpool instruction
        debug!("Would close position");
        Ok(())
    }

    /// Opens a new position.
    async fn open_position(
        &self,
        _pool: &Pubkey,
        tick_lower: i32,
        tick_upper: i32,
    ) -> anyhow::Result<Pubkey> {
        // TODO: Implement actual position open via Whirlpool instruction
        debug!(
            tick_lower = tick_lower,
            tick_upper = tick_upper,
            "Would open position"
        );
        Ok(Pubkey::new_unique())
    }

    /// Increases liquidity in a position.
    async fn increase_liquidity(
        &self,
        _position: &Pubkey,
        liquidity: u128,
    ) -> anyhow::Result<u128> {
        // TODO: Implement actual liquidity increase via Whirlpool instruction
        debug!(liquidity = liquidity, "Would increase liquidity");
        Ok(liquidity)
    }
}

/// Result of profitability check.
#[derive(Debug, Clone)]
pub struct ProfitabilityCheck {
    /// Whether rebalance is profitable.
    pub is_profitable: bool,
    /// Estimated transaction cost in lamports.
    pub estimated_tx_cost: u64,
    /// Expected benefit in USD.
    pub expected_benefit: Decimal,
    /// Minimum required benefit.
    pub min_required_benefit: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rebalance_config_default() {
        let config = RebalanceConfig::default();
        assert_eq!(config.max_slippage_bps, 50);
        assert!(config.collect_fees_first);
    }
}
