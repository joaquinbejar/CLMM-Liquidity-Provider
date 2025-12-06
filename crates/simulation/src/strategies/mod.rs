//! Rebalancing strategies for LP positions.
//!
//! This module provides different strategies for managing LP positions,
//! including when and how to rebalance based on market conditions.

mod periodic;
mod static_range;
mod threshold;

pub use periodic::PeriodicRebalance;
pub use static_range::StaticRange;
pub use threshold::ThresholdRebalance;

use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;

/// Action to take based on strategy evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum RebalanceAction {
    /// Hold current position, no action needed.
    Hold,
    /// Rebalance to a new price range.
    Rebalance {
        /// The new price range to rebalance to.
        new_range: PriceRange,
        /// Reason for the rebalance.
        reason: RebalanceReason,
    },
    /// Close the position entirely.
    Close {
        /// Reason for closing.
        reason: RebalanceReason,
    },
}

/// Reason for a rebalance action.
#[derive(Debug, Clone, PartialEq)]
pub enum RebalanceReason {
    /// Periodic rebalance triggered by time.
    Periodic {
        /// Steps since last rebalance.
        steps_elapsed: u64,
    },
    /// Price moved beyond threshold.
    PriceThreshold {
        /// The price movement percentage that triggered rebalance.
        price_change_pct: Decimal,
    },
    /// Price is out of range.
    OutOfRange {
        /// Current price.
        current_price: Decimal,
    },
    /// Impermanent loss exceeded threshold.
    ILThreshold {
        /// Current IL percentage.
        il_pct: Decimal,
    },
    /// Manual or other reason.
    Manual,
}

/// Context provided to strategies for decision making.
#[derive(Debug, Clone)]
pub struct StrategyContext {
    /// Current price.
    pub current_price: Price,
    /// Current position range.
    pub current_range: PriceRange,
    /// Entry price when position was opened.
    pub entry_price: Price,
    /// Steps since position was opened.
    pub steps_since_open: u64,
    /// Steps since last rebalance.
    pub steps_since_rebalance: u64,
    /// Current impermanent loss percentage.
    pub current_il_pct: Decimal,
    /// Total fees earned so far.
    pub total_fees_earned: Decimal,
}

impl StrategyContext {
    /// Checks if the current price is within the position range.
    #[must_use]
    pub fn is_in_range(&self) -> bool {
        self.current_price.value >= self.current_range.lower_price.value
            && self.current_price.value <= self.current_range.upper_price.value
    }

    /// Calculates the price change percentage from entry.
    #[must_use]
    pub fn price_change_from_entry(&self) -> Decimal {
        if self.entry_price.value == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.current_price.value - self.entry_price.value) / self.entry_price.value
    }

    /// Calculates the price change percentage from range midpoint.
    #[must_use]
    pub fn price_change_from_midpoint(&self) -> Decimal {
        let midpoint = (self.current_range.lower_price.value
            + self.current_range.upper_price.value)
            / Decimal::from(2);
        if midpoint == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.current_price.value - midpoint) / midpoint
    }
}

/// Trait for rebalancing strategies.
pub trait RebalanceStrategy: Send + Sync {
    /// Evaluates the current context and returns the recommended action.
    fn evaluate(&self, context: &StrategyContext) -> RebalanceAction;

    /// Returns the name of the strategy.
    fn name(&self) -> &'static str;

    /// Calculates a new range centered around the current price.
    fn calculate_new_range(&self, current_price: Price, range_width_pct: Decimal) -> PriceRange {
        let half_width = current_price.value * range_width_pct / Decimal::from(2);
        PriceRange::new(
            Price::new(current_price.value - half_width),
            Price::new(current_price.value + half_width),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_context() -> StrategyContext {
        StrategyContext {
            current_price: Price::new(dec!(100)),
            current_range: PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            entry_price: Price::new(dec!(100)),
            steps_since_open: 10,
            steps_since_rebalance: 5,
            current_il_pct: dec!(-0.02),
            total_fees_earned: dec!(50),
        }
    }

    #[test]
    fn test_context_is_in_range() {
        let ctx = create_test_context();
        assert!(ctx.is_in_range());

        let mut ctx_out = ctx.clone();
        ctx_out.current_price = Price::new(dec!(120));
        assert!(!ctx_out.is_in_range());
    }

    #[test]
    fn test_price_change_from_entry() {
        let mut ctx = create_test_context();
        ctx.current_price = Price::new(dec!(110));
        assert_eq!(ctx.price_change_from_entry(), dec!(0.1));
    }
}
