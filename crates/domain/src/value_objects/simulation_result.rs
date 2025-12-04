use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub final_position_value: Decimal,
    pub total_fees_earned: Decimal,
    pub total_il: Decimal,
    pub net_pnl: Decimal,
    pub max_drawdown: Decimal,
    pub time_in_range_percentage: Decimal,
    pub sharpe_ratio: Option<Decimal>,
}
