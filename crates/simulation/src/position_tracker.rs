//! Position tracking for simulations.
//!
//! This module provides functionality to track position state over time,
//! recording snapshots and computing metrics at each step.

use crate::strategies::{RebalanceAction, RebalanceStrategy, StrategyContext};
use clmm_lp_domain::metrics::impermanent_loss::calculate_il_concentrated;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;

/// A snapshot of position state at a point in time.
#[derive(Debug, Clone)]
pub struct PositionSnapshot {
    /// Step number in the simulation.
    pub step: u64,
    /// Current price at this step.
    pub price: Price,
    /// Current position range.
    pub range: PriceRange,
    /// Whether price is in range.
    pub in_range: bool,
    /// Cumulative fees earned up to this step.
    pub cumulative_fees: Decimal,
    /// Current impermanent loss percentage.
    pub il_pct: Decimal,
    /// Current position value in USD.
    pub position_value_usd: Decimal,
    /// Net PnL at this step.
    pub net_pnl: Decimal,
    /// Action taken at this step (if any).
    pub action: Option<RebalanceAction>,
}

/// Tracks position state throughout a simulation.
#[derive(Debug)]
pub struct PositionTracker {
    /// Initial capital in USD.
    pub initial_capital: Decimal,
    /// Entry price.
    pub entry_price: Price,
    /// Current range.
    pub current_range: PriceRange,
    /// All recorded snapshots.
    pub snapshots: Vec<PositionSnapshot>,
    /// Steps since last rebalance.
    pub steps_since_rebalance: u64,
    /// Total rebalance count.
    pub rebalance_count: u32,
    /// Total transaction costs from rebalancing.
    pub total_rebalance_cost: Decimal,
    /// Cost per rebalance in USD.
    pub rebalance_cost: Decimal,
    /// Cumulative fees earned.
    cumulative_fees: Decimal,
    /// Current step.
    current_step: u64,
}

impl PositionTracker {
    /// Creates a new position tracker.
    ///
    /// # Arguments
    ///
    /// * `initial_capital` - Starting capital in USD
    /// * `entry_price` - Price at position entry
    /// * `initial_range` - Initial price range
    /// * `rebalance_cost` - Cost per rebalance transaction in USD
    #[must_use]
    pub fn new(
        initial_capital: Decimal,
        entry_price: Price,
        initial_range: PriceRange,
        rebalance_cost: Decimal,
    ) -> Self {
        Self {
            initial_capital,
            entry_price,
            current_range: initial_range,
            snapshots: Vec::new(),
            steps_since_rebalance: 0,
            rebalance_count: 0,
            total_rebalance_cost: Decimal::ZERO,
            rebalance_cost,
            cumulative_fees: Decimal::ZERO,
            current_step: 0,
        }
    }

    /// Records a step in the simulation.
    ///
    /// # Arguments
    ///
    /// * `price` - Current price
    /// * `step_fees` - Fees earned this step
    /// * `strategy` - Optional strategy to evaluate for rebalancing
    ///
    /// # Returns
    ///
    /// The action taken (if any)
    pub fn record_step<S: RebalanceStrategy>(
        &mut self,
        price: Price,
        step_fees: Decimal,
        strategy: Option<&S>,
    ) -> Option<RebalanceAction> {
        self.current_step += 1;
        self.steps_since_rebalance += 1;
        self.cumulative_fees += step_fees;

        // Calculate current IL
        let il_pct = calculate_il_concentrated(
            self.entry_price.value,
            price.value,
            self.current_range.lower_price.value,
            self.current_range.upper_price.value,
        )
        .unwrap_or(Decimal::ZERO);

        // Calculate position value
        let il_amount = self.initial_capital * il_pct;
        let position_value =
            self.initial_capital + il_amount + self.cumulative_fees - self.total_rebalance_cost;
        let net_pnl = position_value - self.initial_capital;

        // Check if in range
        let in_range = price.value >= self.current_range.lower_price.value
            && price.value <= self.current_range.upper_price.value;

        // Evaluate strategy if provided
        let action = strategy.map(|s| {
            let context = StrategyContext {
                current_price: price,
                current_range: self.current_range.clone(),
                entry_price: self.entry_price,
                steps_since_open: self.current_step,
                steps_since_rebalance: self.steps_since_rebalance,
                current_il_pct: il_pct,
                total_fees_earned: self.cumulative_fees,
            };
            s.evaluate(&context)
        });

        // Handle rebalance action
        let final_action = if let Some(ref act) = action {
            match act {
                RebalanceAction::Rebalance { new_range, .. } => {
                    self.execute_rebalance(new_range.clone());
                    action.clone()
                }
                RebalanceAction::Close { .. } => action.clone(),
                RebalanceAction::Hold => None,
            }
        } else {
            None
        };

        // Record snapshot
        let snapshot = PositionSnapshot {
            step: self.current_step,
            price,
            range: self.current_range.clone(),
            in_range,
            cumulative_fees: self.cumulative_fees,
            il_pct,
            position_value_usd: position_value,
            net_pnl,
            action: final_action.clone(),
        };
        self.snapshots.push(snapshot);

        final_action
    }

    /// Executes a rebalance to a new range.
    fn execute_rebalance(&mut self, new_range: PriceRange) {
        self.current_range = new_range;
        self.steps_since_rebalance = 0;
        self.rebalance_count += 1;
        self.total_rebalance_cost += self.rebalance_cost;
    }

    /// Returns summary statistics for the tracked position.
    #[must_use]
    pub fn summary(&self) -> TrackerSummary {
        let total_steps = self.snapshots.len() as u64;
        let in_range_steps = self.snapshots.iter().filter(|s| s.in_range).count() as u64;

        let time_in_range_pct = if total_steps > 0 {
            Decimal::from(in_range_steps) / Decimal::from(total_steps)
        } else {
            Decimal::ZERO
        };

        let final_snapshot = self.snapshots.last();
        let final_value = final_snapshot
            .map(|s| s.position_value_usd)
            .unwrap_or(self.initial_capital);
        let final_pnl = final_snapshot.map(|s| s.net_pnl).unwrap_or(Decimal::ZERO);
        let final_il = final_snapshot.map(|s| s.il_pct).unwrap_or(Decimal::ZERO);

        // Calculate max drawdown
        let mut peak = self.initial_capital;
        let mut max_drawdown = Decimal::ZERO;
        for snapshot in &self.snapshots {
            if snapshot.position_value_usd > peak {
                peak = snapshot.position_value_usd;
            }
            let drawdown = (peak - snapshot.position_value_usd) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Calculate HODL comparison
        let hodl_value = if let Some(final_snap) = final_snapshot {
            // Simple HODL: assume 50/50 split at entry, track price change
            let price_ratio = final_snap.price.value / self.entry_price.value;
            // HODL value = initial * (1 + price_change) / 2 + initial / 2
            // Simplified: assume quote token is stable
            self.initial_capital * (Decimal::ONE + price_ratio) / Decimal::from(2)
        } else {
            self.initial_capital
        };
        let vs_hodl = final_value - hodl_value;

        TrackerSummary {
            total_steps,
            final_value,
            final_pnl,
            final_il_pct: final_il,
            total_fees: self.cumulative_fees,
            time_in_range_pct,
            rebalance_count: self.rebalance_count,
            total_rebalance_cost: self.total_rebalance_cost,
            max_drawdown,
            hodl_value,
            vs_hodl,
        }
    }
}

/// Summary statistics from position tracking.
#[derive(Debug, Clone)]
pub struct TrackerSummary {
    /// Total simulation steps.
    pub total_steps: u64,
    /// Final position value in USD.
    pub final_value: Decimal,
    /// Final net PnL.
    pub final_pnl: Decimal,
    /// Final impermanent loss percentage.
    pub final_il_pct: Decimal,
    /// Total fees earned.
    pub total_fees: Decimal,
    /// Percentage of time in range.
    pub time_in_range_pct: Decimal,
    /// Number of rebalances executed.
    pub rebalance_count: u32,
    /// Total cost of rebalancing.
    pub total_rebalance_cost: Decimal,
    /// Maximum drawdown percentage.
    pub max_drawdown: Decimal,
    /// HODL strategy value for comparison.
    pub hodl_value: Decimal,
    /// Performance vs HODL (positive = outperformed).
    pub vs_hodl: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::StaticRange;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tracker_basic() {
        let mut tracker = PositionTracker::new(
            dec!(1000),
            Price::new(dec!(100)),
            PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            dec!(5),
        );

        // Record some steps
        tracker.record_step::<StaticRange>(Price::new(dec!(100)), dec!(10), None);
        tracker.record_step::<StaticRange>(Price::new(dec!(102)), dec!(10), None);
        tracker.record_step::<StaticRange>(Price::new(dec!(98)), dec!(10), None);

        assert_eq!(tracker.snapshots.len(), 3);
        assert_eq!(tracker.cumulative_fees, dec!(30));

        let summary = tracker.summary();
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.total_fees, dec!(30));
        assert_eq!(summary.rebalance_count, 0);
    }

    #[test]
    fn test_tracker_with_strategy() {
        use crate::strategies::ThresholdRebalance;

        let mut tracker = PositionTracker::new(
            dec!(1000),
            Price::new(dec!(100)),
            PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            dec!(5),
        );

        let strategy = ThresholdRebalance::new(dec!(0.05), dec!(0.2));

        // Price stays in range, no rebalance
        tracker.record_step(Price::new(dec!(100)), dec!(10), Some(&strategy));
        assert_eq!(tracker.rebalance_count, 0);

        // Price moves significantly, should trigger rebalance
        tracker.record_step(Price::new(dec!(120)), dec!(5), Some(&strategy));
        assert_eq!(tracker.rebalance_count, 1);
        assert_eq!(tracker.total_rebalance_cost, dec!(5));

        // New range should be centered on 120
        assert_eq!(tracker.current_range.lower_price.value, dec!(108)); // 120 - 12
        assert_eq!(tracker.current_range.upper_price.value, dec!(132)); // 120 + 12
    }

    #[test]
    fn test_tracker_time_in_range() {
        let mut tracker = PositionTracker::new(
            dec!(1000),
            Price::new(dec!(100)),
            PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            dec!(5),
        );

        // 2 in range, 1 out of range
        tracker.record_step::<StaticRange>(Price::new(dec!(100)), dec!(10), None);
        tracker.record_step::<StaticRange>(Price::new(dec!(120)), dec!(0), None); // out
        tracker.record_step::<StaticRange>(Price::new(dec!(105)), dec!(10), None);

        let summary = tracker.summary();
        // 2/3 in range
        assert!(summary.time_in_range_pct > dec!(0.66));
        assert!(summary.time_in_range_pct < dec!(0.67));
    }
}
