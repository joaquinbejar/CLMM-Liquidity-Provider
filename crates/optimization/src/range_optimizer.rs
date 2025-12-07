use crate::objective::ObjectiveFunction;
use clmm_lp_domain::entities::position::Position;
use clmm_lp_domain::value_objects::OptimizationResult;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use clmm_lp_domain::value_objects::simulation_result::SimulationResult;
use clmm_lp_simulation::liquidity::ConstantLiquidity;
use clmm_lp_simulation::monte_carlo::MonteCarloRunner;
use clmm_lp_simulation::volume::ConstantVolume;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Optimizer for finding the best price range.
pub struct RangeOptimizer {
    /// Number of iterations for Monte Carlo.
    pub iterations: usize,
    /// Number of steps per iteration.
    pub steps: usize,
    /// Time step in years.
    pub time_step: f64,
}

impl RangeOptimizer {
    /// Creates a new RangeOptimizer.
    pub fn new(iterations: usize, steps: usize, time_step: f64) -> Self {
        Self {
            iterations,
            steps,
            time_step,
        }
    }

    /// Optimizes the price range for a given position.
    #[allow(clippy::too_many_arguments)]
    pub fn optimize<O: ObjectiveFunction>(
        &self,
        base_position: Position,
        current_price: Decimal,
        volatility: f64,
        drift: f64,
        volume: ConstantVolume,
        pool_liquidity: u128,
        fee_rate: Decimal,
        objective: O,
    ) -> OptimizationResult {
        // Candidate widths: 1%, 2%, 5%, 10%, 20%, 50%
        let widths = vec![0.01, 0.02, 0.05, 0.10, 0.20, 0.50];

        let mut best_result: Option<(SimulationResult, PriceRange)> = None;
        let mut best_score = Decimal::MIN;

        // Assume 1000 USD capital for estimation
        let _capital = Decimal::from(1000);
        let liquidity_model = ConstantLiquidity::new(pool_liquidity);

        for width in widths {
            let lower_mult = Decimal::from_f64(1.0 - width).unwrap();
            let upper_mult = Decimal::from_f64(1.0 + width).unwrap();

            let lower_price = current_price * lower_mult;
            let upper_price = current_price * upper_mult;

            let range = PriceRange::new(Price::new(lower_price), Price::new(upper_price));

            // Estimate Liquidity L for this range given Capital
            // Narrower range -> Higher L
            // Approximation: L = Capital / (Width_factor)
            // For simplicity, let's use L = 1 / width (relative to 1000 base)
            // Real calc is complex, this proxy ensures narrower ranges get higher fees.
            let width_dec = Decimal::from_f64(width).unwrap();
            let liquidity_proxy = (Decimal::from(1000) / width_dec).to_u128().unwrap_or(1000);

            let mut candidate_position = base_position.clone();
            candidate_position.range = Some(range.clone());
            candidate_position.liquidity_amount = liquidity_proxy;

            let mut runner = MonteCarloRunner {
                position: candidate_position,
                volume_model: volume.clone(),
                liquidity_model: liquidity_model.clone(),
                fee_rate,
                initial_price: current_price,
                drift,
                volatility,
                time_step: self.time_step,
                steps: self.steps,
                iterations: self.iterations,
            };

            let agg_result = runner.run();

            let sim_result = SimulationResult {
                final_position_value: Decimal::ZERO,
                total_fees_earned: agg_result.mean_fees,
                total_il: agg_result.mean_il,
                net_pnl: agg_result.mean_net_pnl,
                max_drawdown: Decimal::ZERO,
                time_in_range_percentage: Decimal::ZERO,
                sharpe_ratio: None,
            };

            let score = objective.evaluate(&sim_result);

            if score > best_score {
                best_score = score;
                best_result = Some((sim_result, range));
            }
        }

        let (best_sim, best_range) = best_result.expect("No candidates evaluated");

        OptimizationResult {
            recommended_range: best_range,
            expected_pnl: best_sim.net_pnl,
            expected_fees: best_sim.total_fees_earned,
            expected_il: best_sim.total_il,
            sharpe_ratio: best_sim.sharpe_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objective::MaximizeNetPnL;
    use clmm_lp_domain::entities::position::{Position, PositionId};
    use clmm_lp_domain::enums::PositionStatus;
    use clmm_lp_domain::value_objects::amount::Amount;
    use primitive_types::U256;
    use uuid::Uuid;

    fn create_dummy_position() -> Position {
        Position {
            id: PositionId(Uuid::new_v4()),
            pool_address: "pool1".to_string(),
            owner_address: "owner1".to_string(),
            liquidity_amount: 0, // will be overwritten
            deposited_amount_a: Amount::new(U256::zero(), 6),
            deposited_amount_b: Amount::new(U256::zero(), 6),
            current_amount_a: Amount::new(U256::zero(), 6),
            current_amount_b: Amount::new(U256::zero(), 6),
            unclaimed_fees_a: Amount::new(U256::zero(), 6),
            unclaimed_fees_b: Amount::new(U256::zero(), 6),
            range: None,
            opened_at: 0,
            status: PositionStatus::Open,
        }
    }

    #[test]
    fn test_optimization_recommends_range() {
        let optimizer = RangeOptimizer::new(10, 5, 1.0 / 365.0);
        let position = create_dummy_position();
        let volume = ConstantVolume::from_amount(Amount::new(U256::from(1000000), 6));
        let current_price = Decimal::from(100);
        let pool_liquidity = 100_000_000;
        let fee_rate = Decimal::from_f64(0.003).unwrap();

        // Low volatility, optimize for PnL
        // Should prefer narrower range because fees > IL (with high volume/liquidity ratio assumption)
        // But our fee model is simple.
        let result = optimizer.optimize(
            position,
            current_price,
            0.1, // 10% vol
            0.0,
            volume,
            pool_liquidity,
            fee_rate,
            MaximizeNetPnL,
        );

        assert!(result.expected_pnl > Decimal::MIN);
        // Check recommended range is valid
        assert!(result.recommended_range.lower_price.value < current_price);
        assert!(result.recommended_range.upper_price.value > current_price);
    }
}
