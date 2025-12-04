use crate::token::{Price, TokenAmount};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub lower_price: Price,
    pub upper_price: Price,
    pub lower_tick: Option<i32>,
    pub upper_tick: Option<i32>,
}

impl Range {
    pub fn is_in_range(&self, current_price: Price) -> bool {
        current_price.0 >= self.lower_price.0 && current_price.0 <= self.upper_price.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub id: PositionId,
    pub pool_id: String,
    pub range: Range,
    pub liquidity: u128,
    pub amount0: TokenAmount,
    pub amount1: TokenAmount,
    pub fee_growth_inside0_last: U256,
    pub fee_growth_inside1_last: U256,
}

impl LiquidityPosition {
    /// Calculates the share of fees this position earns relative to total liquidity at the current tick.
    /// Returns a decimal between 0 and 1.
    pub fn calculate_fee_share(&self, total_liquidity_at_tick: u128) -> Decimal {
        if total_liquidity_at_tick == 0 {
            return Decimal::ZERO;
        }
        // Handle potential overflow if u128 is max, but Decimal supports high precision.
        Decimal::from(self.liquidity) / Decimal::from(total_liquidity_at_tick)
    }
}

use primitive_types::U256;

