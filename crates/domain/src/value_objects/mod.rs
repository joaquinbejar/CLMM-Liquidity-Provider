pub mod amount;
pub mod optimization_result;
pub mod percentage;
pub mod price;
pub mod price_range;
pub mod simulation_result;

pub use optimization_result::OptimizationResult;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityEstimate {
    pub annualized_volatility: Decimal,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpermanentLossResult {
    pub il_percentage: Decimal,
    pub il_amount_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEarnings {
    pub amount_a: Decimal,
    pub amount_b: Decimal,
    pub total_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub tvl_usd: Decimal,
    pub volume_24h_usd: Decimal,
    pub fee_apr_24h: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_95: Decimal,
    pub max_drawdown: Decimal,
}
