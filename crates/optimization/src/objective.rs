use clmm_lp_domain::value_objects::simulation_result::SimulationResult;
use rust_decimal::Decimal;
use std::cmp::Ordering;

/// Trait for objective functions.
pub trait ObjectiveFunction {
    /// Evaluates the simulation result and returns a score.
    fn evaluate(&self, result: &SimulationResult) -> Decimal;
    /// Compares two simulation results.
    fn compare(&self, a: &SimulationResult, b: &SimulationResult) -> Ordering {
        self.evaluate(a)
            .partial_cmp(&self.evaluate(b))
            .unwrap_or(Ordering::Equal)
    }
}

/// Objective function to maximize Net PnL.
pub struct MaximizeNetPnL;

impl ObjectiveFunction for MaximizeNetPnL {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.net_pnl
    }
}

/// Objective function to maximize Fees.
pub struct MaximizeFees;

impl ObjectiveFunction for MaximizeFees {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.total_fees_earned
    }
}

/// Objective function to maximize Sharpe Ratio.
pub struct MaximizeSharpeRatio {
    /// Risk-free rate.
    pub risk_free_rate: Decimal,
}

impl MaximizeSharpeRatio {
    /// Creates a new MaximizeSharpeRatio objective.
    #[must_use]
    pub fn new(risk_free_rate: Decimal) -> Self {
        Self { risk_free_rate }
    }
}

impl ObjectiveFunction for MaximizeSharpeRatio {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        // Very simplified Sharpe: Return / MaxDrawdown (Sortino-ish) or just Return if risk is handled elsewhere.
        // The simulation result struct has sharpe_ratio field if calculated by runner.
        // If not, we fall back to PnL.
        result.sharpe_ratio.unwrap_or(result.net_pnl)
    }
}
