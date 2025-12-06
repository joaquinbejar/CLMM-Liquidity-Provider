//! Threshold-based rebalancing strategy.
//!
//! This strategy rebalances when the price moves beyond a certain
//! threshold from the range midpoint or entry price.

use super::{RebalanceAction, RebalanceReason, RebalanceStrategy, StrategyContext};
use rust_decimal::Decimal;

/// Threshold-based rebalancing strategy.
///
/// Rebalances when price moves beyond a specified percentage threshold
/// from the range midpoint. This is a reactive strategy that responds
/// to significant price movements.
#[derive(Debug, Clone)]
pub struct ThresholdRebalance {
    /// Price movement threshold as a decimal (e.g., 0.05 for 5%).
    pub threshold_pct: Decimal,
    /// Width of the new range as a percentage of current price.
    pub range_width_pct: Decimal,
    /// Whether to also rebalance when out of range (regardless of threshold).
    pub rebalance_on_out_of_range: bool,
    /// Maximum IL before closing position (None = no limit).
    pub max_il_pct: Option<Decimal>,
}

impl ThresholdRebalance {
    /// Creates a new threshold rebalance strategy.
    ///
    /// # Arguments
    ///
    /// * `threshold_pct` - Price movement threshold (0.05 = 5%)
    /// * `range_width_pct` - New range width as percentage (0.2 = 20%)
    #[must_use]
    pub fn new(threshold_pct: Decimal, range_width_pct: Decimal) -> Self {
        Self {
            threshold_pct,
            range_width_pct,
            rebalance_on_out_of_range: true,
            max_il_pct: None,
        }
    }

    /// Sets whether to rebalance when price is out of range.
    #[must_use]
    pub fn rebalance_on_out_of_range(mut self, value: bool) -> Self {
        self.rebalance_on_out_of_range = value;
        self
    }

    /// Sets maximum IL threshold before closing position.
    #[must_use]
    pub fn with_max_il(mut self, max_il_pct: Decimal) -> Self {
        self.max_il_pct = Some(max_il_pct);
        self
    }
}

impl RebalanceStrategy for ThresholdRebalance {
    fn evaluate(&self, context: &StrategyContext) -> RebalanceAction {
        // Check IL limit first
        if let Some(max_il) = self.max_il_pct {
            // IL is negative, so we compare absolute values
            if context.current_il_pct.abs() > max_il.abs() {
                return RebalanceAction::Close {
                    reason: RebalanceReason::ILThreshold {
                        il_pct: context.current_il_pct,
                    },
                };
            }
        }

        // Check if out of range
        if !context.is_in_range() && self.rebalance_on_out_of_range {
            let new_range = self.calculate_new_range(context.current_price, self.range_width_pct);
            return RebalanceAction::Rebalance {
                new_range,
                reason: RebalanceReason::OutOfRange {
                    current_price: context.current_price.value,
                },
            };
        }

        // Check price movement from midpoint
        let price_change = context.price_change_from_midpoint().abs();
        if price_change >= self.threshold_pct {
            let new_range = self.calculate_new_range(context.current_price, self.range_width_pct);
            return RebalanceAction::Rebalance {
                new_range,
                reason: RebalanceReason::PriceThreshold {
                    price_change_pct: price_change,
                },
            };
        }

        RebalanceAction::Hold
    }

    fn name(&self) -> &'static str {
        "Threshold Rebalance"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clmm_lp_domain::value_objects::price::Price;
    use clmm_lp_domain::value_objects::price_range::PriceRange;
    use rust_decimal_macros::dec;

    fn create_context(current_price: Decimal, il_pct: Decimal) -> StrategyContext {
        StrategyContext {
            current_price: Price::new(current_price),
            current_range: PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            entry_price: Price::new(dec!(100)),
            steps_since_open: 100,
            steps_since_rebalance: 50,
            current_il_pct: il_pct,
            total_fees_earned: dec!(50),
        }
    }

    #[test]
    fn test_threshold_holds_within_threshold() {
        let strategy = ThresholdRebalance::new(dec!(0.05), dec!(0.2));
        // Price at 100, midpoint is 100, no change
        let ctx = create_context(dec!(100), dec!(-0.01));
        assert_eq!(strategy.evaluate(&ctx), RebalanceAction::Hold);
    }

    #[test]
    fn test_threshold_rebalances_on_price_move() {
        let strategy = ThresholdRebalance::new(dec!(0.05), dec!(0.2));
        // Price at 108, midpoint is 100, 8% change > 5% threshold
        let ctx = create_context(dec!(108), dec!(-0.02));

        match strategy.evaluate(&ctx) {
            RebalanceAction::Rebalance { reason, .. } => {
                assert!(matches!(reason, RebalanceReason::PriceThreshold { .. }));
            }
            _ => panic!("Expected Rebalance action"),
        }
    }

    #[test]
    fn test_threshold_rebalances_on_out_of_range() {
        let strategy = ThresholdRebalance::new(dec!(0.10), dec!(0.2));
        // Price at 120, out of range 90-110
        let ctx = create_context(dec!(120), dec!(-0.03));

        match strategy.evaluate(&ctx) {
            RebalanceAction::Rebalance { reason, .. } => {
                assert!(matches!(reason, RebalanceReason::OutOfRange { .. }));
            }
            _ => panic!("Expected Rebalance action"),
        }
    }

    #[test]
    fn test_threshold_closes_on_max_il() {
        let strategy = ThresholdRebalance::new(dec!(0.05), dec!(0.2)).with_max_il(dec!(0.10));
        // IL at -15%, exceeds -10% max
        let ctx = create_context(dec!(100), dec!(-0.15));

        match strategy.evaluate(&ctx) {
            RebalanceAction::Close { reason } => {
                assert!(matches!(reason, RebalanceReason::ILThreshold { .. }));
            }
            _ => panic!("Expected Close action"),
        }
    }

    #[test]
    fn test_threshold_no_rebalance_on_out_of_range_disabled() {
        let strategy =
            ThresholdRebalance::new(dec!(0.50), dec!(0.2)).rebalance_on_out_of_range(false);
        // Price at 120, out of range, but feature disabled and threshold not met
        let ctx = create_context(dec!(120), dec!(-0.03));
        // Midpoint is 100, price is 120, that's 20% change which is < 50% threshold
        assert_eq!(strategy.evaluate(&ctx), RebalanceAction::Hold);
    }
}
