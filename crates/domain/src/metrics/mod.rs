use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub mod fees;
pub mod impermanent_loss;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpermanentLoss {
    pub absolute_loss_usd: Decimal,
    pub percentage_loss: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APY {
    pub estimated_annual_return: Decimal,
    pub based_on_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnL {
    pub unrealized_pnl_usd: Decimal,
    pub realized_pnl_usd: Decimal,
    pub total_pnl_usd: Decimal,
    pub roi_percent: Decimal,
}
