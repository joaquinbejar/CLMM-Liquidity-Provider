use crate::liquidity::LiquidityModel;
use crate::price_path::PricePathGenerator;
use crate::volume::VolumeModel;
use clmm_lp_domain::entities::position::Position;
use clmm_lp_domain::metrics::impermanent_loss::calculate_il_concentrated;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::simulation_result::SimulationResult;
use rust_decimal::Decimal;

/// Engine for running simulations.
pub struct SimulationEngine<P: PricePathGenerator, V: VolumeModel, L: LiquidityModel> {
    /// The position to simulate.
    pub position: Position,
    /// The price path generator.
    pub price_path_generator: P,
    /// The volume model.
    pub volume_model: V,
    /// The liquidity model for global pool liquidity.
    pub liquidity_model: L,
    /// The pool fee rate (e.g., 0.003 for 0.3%).
    pub fee_rate: Decimal,
    /// The number of simulation steps.
    pub steps: usize,
}

impl<P: PricePathGenerator, V: VolumeModel, L: LiquidityModel> SimulationEngine<P, V, L> {
    /// Creates a new SimulationEngine.
    pub fn new(
        position: Position,
        price_path_generator: P,
        volume_model: V,
        liquidity_model: L,
        fee_rate: Decimal,
        steps: usize,
    ) -> Self {
        Self {
            position,
            price_path_generator,
            volume_model,
            liquidity_model,
            fee_rate,
            steps,
        }
    }

    /// Runs the simulation.
    pub fn run(&mut self) -> SimulationResult {
        let prices = self.price_path_generator.generate(self.steps);

        let mut total_fees_usd = Decimal::ZERO;
        let initial_price = prices
            .first()
            .cloned()
            .unwrap_or(Price::new(Decimal::ONE))
            .value;
        let mut current_price = initial_price;

        // Initial value (approximate for simulation)
        // Real implementation would calculate exact amounts held at initial price
        let initial_value_usd = Decimal::from(1000); // Placeholder, should compute from position.liquidity

        // We assume position range is fixed for this basic simulation
        let range = self
            .position
            .range
            .clone()
            .expect("CLMM position needs range");
        let lower = range.lower_price.value;
        let upper = range.upper_price.value;

        let mut time_in_range_count = 0;

        for price in &prices {
            current_price = price.value;

            // 1. Check range
            let in_range = current_price >= lower && current_price <= upper;
            if in_range {
                time_in_range_count += 1;

                // 2. Accrue fees
                // Fee logic: Fee Share * Volume * FeeRate
                // Simplified: Fixed daily volume / steps per day
                let vol = self.volume_model.next_volume().to_decimal();

                // Get global liquidity at current price
                let global_liquidity = self.liquidity_model.get_liquidity_at_price(current_price);

                // Calculate fee share
                let fee_share = if global_liquidity > 0 {
                    let pos_liq = Decimal::from(self.position.liquidity_amount);
                    let global_liq = Decimal::from(global_liquidity);
                    // Cap share at 1.0 (100%)
                    if pos_liq > global_liq {
                        Decimal::ONE
                    } else {
                        pos_liq / global_liq
                    }
                } else {
                    Decimal::ZERO
                };

                let step_fees = vol * fee_share * self.fee_rate;
                total_fees_usd += step_fees;
            }
        }

        // 3. Calculate Final IL
        let il_pct = calculate_il_concentrated(initial_price, current_price, lower, upper)
            .unwrap_or(Decimal::ZERO);

        let il_amount = initial_value_usd * il_pct; // Negative value
        // let final_value_hold = initial_value_usd; // Simplified (assuming stable quote or normalized)
        let final_position_value = initial_value_usd + il_amount + total_fees_usd;
        let net_pnl = final_position_value - initial_value_usd;

        SimulationResult {
            final_position_value,
            total_fees_earned: total_fees_usd,
            total_il: il_amount,
            net_pnl,
            max_drawdown: Decimal::ZERO, // Need track path for this
            time_in_range_percentage: Decimal::from(time_in_range_count)
                / Decimal::from(self.steps),
            sharpe_ratio: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liquidity::ConstantLiquidity;
    use crate::price_path::DeterministicPricePath;
    use crate::volume::ConstantVolume;
    use clmm_lp_domain::entities::position::Position;
    use clmm_lp_domain::enums::PositionStatus;
    use clmm_lp_domain::value_objects::price::Price;
    use clmm_lp_domain::value_objects::{amount::Amount, price_range::PriceRange};
    use primitive_types::U256;
    use rust_decimal::prelude::FromPrimitive;
    use uuid::Uuid;

    fn create_dummy_position() -> Position {
        Position {
            id: clmm_lp_domain::entities::position::PositionId(Uuid::new_v4()),
            pool_address: "pool1".to_string(),
            owner_address: "owner1".to_string(),
            liquidity_amount: 1000,
            deposited_amount_a: Amount::new(U256::zero(), 6),
            deposited_amount_b: Amount::new(U256::zero(), 6),
            current_amount_a: Amount::new(U256::zero(), 6),
            current_amount_b: Amount::new(U256::zero(), 6),
            unclaimed_fees_a: Amount::new(U256::zero(), 6),
            unclaimed_fees_b: Amount::new(U256::zero(), 6),
            range: Some(PriceRange::new(
                Price::new(Decimal::from(90)),
                Price::new(Decimal::from(110)),
            )),
            opened_at: 0,
            status: PositionStatus::Open,
        }
    }

    #[test]
    fn test_simulation_flat_price() {
        let position = create_dummy_position();
        // 1,000,000 units with 6 decimals = 1,000,000 * 10^6 raw
        let raw_amount = U256::from(1_000_000) * U256::from(1_000_000);
        let volume = ConstantVolume::from_amount(Amount::new(raw_amount, 6));
        let liquidity_model = ConstantLiquidity::new(10000); // 10x position liquidity
        let fee_rate = Decimal::from_f64(0.003).unwrap();

        // Price stays at 100 (in range 90-110)
        let prices = vec![
            Price::new(Decimal::from(100)),
            Price::new(Decimal::from(100)),
            Price::new(Decimal::from(100)),
        ];
        let path_gen = DeterministicPricePath { prices };

        let mut engine =
            SimulationEngine::new(position, path_gen, volume, liquidity_model, fee_rate, 3);
        let result = engine.run();

        // No price change -> 0 IL
        assert_eq!(result.total_il, Decimal::ZERO);
        // Fees should be accumulated
        // Share = 1000 / 10000 = 0.1
        // Vol = 1,000,000
        // Fee Rate = 0.003
        // Fees per step = 1M * 0.1 * 0.003 = 300
        // Total (3 steps) = 900
        assert!(result.total_fees_earned > Decimal::ZERO);
        assert_eq!(result.total_fees_earned, Decimal::from(900));

        // Time in range should be 100% (3/3)
        assert_eq!(result.time_in_range_percentage, Decimal::ONE);
    }

    #[test]
    fn test_simulation_out_of_range() {
        let position = create_dummy_position();
        let volume = ConstantVolume::from_amount(Amount::new(U256::from(1000000), 6));
        let liquidity_model = ConstantLiquidity::new(10000);
        let fee_rate = Decimal::from_f64(0.003).unwrap();

        // Price moves to 120 (out of range 90-110)
        let prices = vec![
            Price::new(Decimal::from(100)),
            Price::new(Decimal::from(120)),
        ];
        let path_gen = DeterministicPricePath { prices };

        let mut engine =
            SimulationEngine::new(position, path_gen, volume, liquidity_model, fee_rate, 2);
        let result = engine.run();

        assert_eq!(
            result.time_in_range_percentage,
            Decimal::from_f64(0.5).unwrap()
        );

        // IL should be negative (price moved)
        assert!(result.total_il < Decimal::ZERO);
    }
}
