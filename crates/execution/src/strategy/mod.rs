//! Strategy execution for automated position management.
//!
//! Provides automated strategy execution including:
//! - Decision engine
//! - Rebalancing logic
//! - Position lifecycle management

mod decision;
mod executor;
mod rebalance;

pub use decision::*;
pub use executor::*;
pub use rebalance::*;

use rust_decimal::Decimal;

/// Decision made by the strategy engine.
#[derive(Debug, Clone)]
pub enum Decision {
    /// Hold current position.
    Hold,
    /// Rebalance to a new range.
    Rebalance {
        /// New lower tick.
        new_tick_lower: i32,
        /// New upper tick.
        new_tick_upper: i32,
    },
    /// Close the position.
    Close,
    /// Increase liquidity.
    IncreaseLiquidity {
        /// Amount to add.
        amount: Decimal,
    },
    /// Decrease liquidity.
    DecreaseLiquidity {
        /// Amount to remove.
        amount: Decimal,
    },
    /// Collect fees.
    CollectFees,
}

impl Decision {
    /// Returns a human-readable description.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Hold => "Hold current position".to_string(),
            Self::Rebalance {
                new_tick_lower,
                new_tick_upper,
            } => {
                format!(
                    "Rebalance to ticks [{}, {}]",
                    new_tick_lower, new_tick_upper
                )
            }
            Self::Close => "Close position".to_string(),
            Self::IncreaseLiquidity { amount } => format!("Increase liquidity by {}", amount),
            Self::DecreaseLiquidity { amount } => format!("Decrease liquidity by {}", amount),
            Self::CollectFees => "Collect accumulated fees".to_string(),
        }
    }

    /// Returns true if this decision requires a transaction.
    #[must_use]
    pub fn requires_transaction(&self) -> bool {
        !matches!(self, Self::Hold)
    }
}
