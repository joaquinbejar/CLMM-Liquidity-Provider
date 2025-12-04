use amm_domain::entities::position::Position;
use amm_domain::value_objects::price::Price;
use amm_domain::value_objects::simulation_result::SimulationResult;
use amm_domain::metrics::impermanent_loss::calculate_il_concentrated;
use crate::price_path::PricePathGenerator;
use crate::volume::VolumeModel;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

pub struct SimulationEngine<P: PricePathGenerator, V: VolumeModel> {
    pub position: Position,
    pub price_path_generator: P,
    pub volume_model: V,
    pub steps: usize,
}

impl<P: PricePathGenerator, V: VolumeModel> SimulationEngine<P, V> {
    pub fn new(position: Position, price_path_generator: P, volume_model: V, steps: usize) -> Self {
        Self {
            position,
            price_path_generator,
            volume_model,
            steps,
        }
    }

    pub fn run(&mut self) -> SimulationResult {
        let prices = self.price_path_generator.generate(self.steps);
        
        let mut total_fees_usd = Decimal::ZERO;
        let initial_price = prices.first().cloned().unwrap_or(Price::new(Decimal::ONE)).value;
        let mut current_price = initial_price;
        
        // Initial value (approximate for simulation)
        // Real implementation would calculate exact amounts held at initial price
        let initial_value_usd = Decimal::from(1000); // Placeholder, should compute from position.liquidity
        
        // We assume position range is fixed for this basic simulation
        let range = self.position.range.clone().expect("CLMM position needs range");
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
                
                // Fee share approx: Liquidity / GlobalLiquidity (unknown here, so we assume a share or standard calc)
                // For now, let's assume we capture 0.1% of volume (very naive)
                // TODO: Inject Pool state to know global liquidity
                let fee_share = Decimal::from_f64(0.001).unwrap(); 
                let fee_rate = Decimal::from_f64(0.003).unwrap(); // 0.3%
                
                let step_fees = vol * fee_share * fee_rate;
                total_fees_usd += step_fees;
            }
        }

        // 3. Calculate Final IL
        let il_pct = calculate_il_concentrated(
            initial_price, 
            current_price, 
            lower, 
            upper
        ).unwrap_or(Decimal::ZERO);
        
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
            time_in_range_percentage: Decimal::from(time_in_range_count) / Decimal::from(self.steps),
            sharpe_ratio: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::price_path::DeterministicPricePath;
    use crate::volume::ConstantVolume;
    use amm_domain::entities::position::Position;
    use amm_domain::value_objects::{amount::Amount, price_range::PriceRange};
    use amm_domain::value_objects::price::Price;
    use amm_domain::enums::PositionStatus;
    use primitive_types::U256;
    use uuid::Uuid;

    fn create_dummy_position() -> Position {
        Position {
            id: amm_domain::entities::position::PositionId(Uuid::new_v4()),
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
                Price::new(Decimal::from(110))
            )),
            opened_at: 0,
            status: PositionStatus::Open,
        }
    }

    #[test]
    fn test_simulation_flat_price() {
        let position = create_dummy_position();
        let volume = ConstantVolume { amount: Amount::new(U256::from(1000000), 6) };
        
        // Price stays at 100 (in range 90-110)
        let prices = vec![
            Price::new(Decimal::from(100)), 
            Price::new(Decimal::from(100)),
            Price::new(Decimal::from(100))
        ];
        let path_gen = DeterministicPricePath { prices };
        
        let mut engine = SimulationEngine::new(position, path_gen, volume, 3);
        let result = engine.run();
        
        // No price change -> 0 IL
        assert_eq!(result.total_il, Decimal::ZERO);
        // Fees should be accumulated
        assert!(result.total_fees_earned > Decimal::ZERO);
        // Time in range should be 100% (3/3)
        assert_eq!(result.time_in_range_percentage, Decimal::ONE);
    }

    #[test]
    fn test_simulation_out_of_range() {
        let position = create_dummy_position();
        let volume = ConstantVolume { amount: Amount::new(U256::from(1000000), 6) };
        
        // Price moves to 120 (out of range 90-110)
        let prices = vec![
            Price::new(Decimal::from(100)), 
            Price::new(Decimal::from(120)),
        ];
        let path_gen = DeterministicPricePath { prices };
        
        let mut engine = SimulationEngine::new(position, path_gen, volume, 2);
        let result = engine.run();
        
        // 1 step in range (100), 1 step out (120). 
        // Logic checks range for each step. If range check is simple inclusive:
        // Step 0: 100 (in). Step 1: 120 (out).
        // Note: The engine currently iterates provided prices.
        // If generator returns 2 prices, we check both? 
        // Usually simulation steps imply transitions. 
        // Here we treat price points as snapshots.
        // 100 is in. 120 is out.
        // So 50% time in range.
        assert_eq!(result.time_in_range_percentage, Decimal::from_f64(0.5).unwrap());
        
        // IL should be negative (price moved)
        assert!(result.total_il < Decimal::ZERO);
    }
}
