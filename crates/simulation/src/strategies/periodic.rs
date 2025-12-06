//! Periodic rebalancing strategy.
//!
//! This strategy rebalances the position at fixed time intervals,
//! regardless of price movements.

use super::{RebalanceAction, RebalanceReason, RebalanceStrategy, StrategyContext};
use rust_decimal::Decimal;

/// Periodic rebalancing strategy.
///
/// Rebalances the position every N steps, centering the new range
/// around the current price.
#[derive(Debug, Clone)]
pub struct PeriodicRebalance {
    /// Number of steps between rebalances.
    pub rebalance_interval: u64,
    /// Width of the range as a percentage of current price (e.g., 0.2 for ±10%).
    pub range_width_pct: Decimal,
    /// Whether to rebalance only when out of range.
    pub only_when_out_of_range: bool,
}

impl PeriodicRebalance {
    /// Creates a new periodic rebalance strategy.
    ///
    /// # Arguments
    ///
    /// * `rebalance_interval` - Number of steps between rebalances
    /// * `range_width_pct` - Total range width as percentage (0.2 = 20% total width)
    #[must_use]
    pub fn new(rebalance_interval: u64, range_width_pct: Decimal) -> Self {
        Self {
            rebalance_interval,
            range_width_pct,
            only_when_out_of_range: false,
        }
    }

    /// Sets whether to only rebalance when price is out of range.
    #[must_use]
    pub fn only_when_out_of_range(mut self, value: bool) -> Self {
        self.only_when_out_of_range = value;
        self
    }
}

impl RebalanceStrategy for PeriodicRebalance {
    fn evaluate(&self, context: &StrategyContext) -> RebalanceAction {
        // Check if it's time to rebalance
        if context.steps_since_rebalance < self.rebalance_interval {
            return RebalanceAction::Hold;
        }

        // If only_when_out_of_range is set, check range
        if self.only_when_out_of_range && context.is_in_range() {
            return RebalanceAction::Hold;
        }

        // Time to rebalance - create new range centered on current price
        let new_range = self.calculate_new_range(context.current_price, self.range_width_pct);

        RebalanceAction::Rebalance {
            new_range,
            reason: RebalanceReason::Periodic {
                steps_elapsed: context.steps_since_rebalance,
            },
        }
    }

    fn name(&self) -> &'static str {
        "Periodic Rebalance"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clmm_lp_domain::value_objects::price::Price;
    use clmm_lp_domain::value_objects::price_range::PriceRange;
    use rust_decimal_macros::dec;

    fn create_context(steps_since_rebalance: u64, current_price: Decimal) -> StrategyContext {
        StrategyContext {
            current_price: Price::new(current_price),
            current_range: PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            entry_price: Price::new(dec!(100)),
            steps_since_open: 100,
            steps_since_rebalance,
            current_il_pct: dec!(-0.02),
            total_fees_earned: dec!(50),
        }
    }

    #[test]
    fn test_periodic_holds_before_interval() {
        let strategy = PeriodicRebalance::new(10, dec!(0.2));
        let ctx = create_context(5, dec!(100));
        assert_eq!(strategy.evaluate(&ctx), RebalanceAction::Hold);
    }

    #[test]
    fn test_periodic_rebalances_at_interval() {
        let strategy = PeriodicRebalance::new(10, dec!(0.2));
        let ctx = create_context(10, dec!(105));

        match strategy.evaluate(&ctx) {
            RebalanceAction::Rebalance { new_range, reason } => {
                // New range should be centered on 105 with 20% width
                // Lower: 105 - (105 * 0.2 / 2) = 105 - 10.5 = 94.5
                // Upper: 105 + 10.5 = 115.5
                assert_eq!(new_range.lower_price.value, dec!(94.5));
                assert_eq!(new_range.upper_price.value, dec!(115.5));
                assert!(matches!(reason, RebalanceReason::Periodic { .. }));
            }
            _ => panic!("Expected Rebalance action"),
        }
    }

    #[test]
    fn test_periodic_only_when_out_of_range() {
        let strategy = PeriodicRebalance::new(10, dec!(0.2)).only_when_out_of_range(true);

        // In range - should hold even at interval
        let ctx_in = create_context(15, dec!(100));
        assert_eq!(strategy.evaluate(&ctx_in), RebalanceAction::Hold);

        // Out of range - should rebalance
        let ctx_out = create_context(15, dec!(120));
        assert!(matches!(
            strategy.evaluate(&ctx_out),
            RebalanceAction::Rebalance { .. }
        ));
    }
}
