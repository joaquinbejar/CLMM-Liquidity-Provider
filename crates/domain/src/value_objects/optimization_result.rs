use crate::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub recommended_range: PriceRange,
    pub expected_pnl: Decimal,
    pub expected_fees: Decimal,
    pub expected_il: Decimal,
    pub sharpe_ratio: Option<Decimal>,
}
