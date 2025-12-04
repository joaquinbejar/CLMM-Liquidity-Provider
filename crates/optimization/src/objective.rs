use amm_domain::value_objects::simulation_result::SimulationResult;
use rust_decimal::Decimal;
use std::cmp::Ordering;

pub trait ObjectiveFunction {
    fn evaluate(&self, result: &SimulationResult) -> Decimal;
    fn compare(&self, a: &SimulationResult, b: &SimulationResult) -> Ordering {
        self.evaluate(a)
            .partial_cmp(&self.evaluate(b))
            .unwrap_or(Ordering::Equal)
    }
}

pub struct MaximizeNetPnL;
impl ObjectiveFunction for MaximizeNetPnL {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.net_pnl
    }
}

pub struct MaximizeFees;
impl ObjectiveFunction for MaximizeFees {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.total_fees_earned
    }
}

pub struct MaximizeSharpeRatio {
    pub risk_free_rate: Decimal,
}
impl ObjectiveFunction for MaximizeSharpeRatio {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        // Very simplified Sharpe: Return / MaxDrawdown (Sortino-ish) or just Return if risk is handled elsewhere.
        // The simulation result struct has sharpe_ratio field if calculated by runner.
        // If not, we fall back to PnL.
        result.sharpe_ratio.unwrap_or(result.net_pnl)
    }
}
