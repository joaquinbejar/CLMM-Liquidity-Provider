//! Static range strategy - no rebalancing.
//!
//! This strategy maintains the initial position range without any rebalancing.
//! It serves as a baseline for comparing other strategies.

use super::{RebalanceAction, RebalanceStrategy, StrategyContext};

/// Static range strategy that never rebalances.
///
/// This is the simplest strategy - set a range and hold it regardless
/// of price movements. Useful as a baseline comparison.
#[derive(Debug, Clone, Default)]
pub struct StaticRange;

impl StaticRange {
    /// Creates a new static range strategy.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl RebalanceStrategy for StaticRange {
    fn evaluate(&self, _context: &StrategyContext) -> RebalanceAction {
        // Never rebalance
        RebalanceAction::Hold
    }

    fn name(&self) -> &'static str {
        "Static Range"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clmm_lp_domain::value_objects::price::Price;
    use clmm_lp_domain::value_objects::price_range::PriceRange;
    use rust_decimal_macros::dec;

    #[test]
    fn test_static_always_holds() {
        let strategy = StaticRange::new();

        // In range
        let ctx = StrategyContext {
            current_price: Price::new(dec!(100)),
            current_range: PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            entry_price: Price::new(dec!(100)),
            steps_since_open: 100,
            steps_since_rebalance: 100,
            current_il_pct: dec!(-0.05),
            total_fees_earned: dec!(100),
        };
        assert_eq!(strategy.evaluate(&ctx), RebalanceAction::Hold);

        // Out of range - still holds
        let ctx_out = StrategyContext {
            current_price: Price::new(dec!(150)),
            ..ctx
        };
        assert_eq!(strategy.evaluate(&ctx_out), RebalanceAction::Hold);
    }
}
