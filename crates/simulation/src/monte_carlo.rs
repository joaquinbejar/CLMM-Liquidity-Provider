use crate::engine::SimulationEngine;
use crate::price_path::GeometricBrownianMotion;
use crate::volume::VolumeModel;
use amm_domain::entities::position::Position;
use amm_domain::value_objects::simulation_result::SimulationResult;
use rust_decimal::Decimal;

pub struct MonteCarloRunner<V: VolumeModel + Clone> {
    pub position: Position,
    pub volume_model: V,
    pub initial_price: Decimal,
    pub drift: f64,
    pub volatility: f64,
    pub time_step: f64,
    pub steps: usize,
    pub iterations: usize,
}

pub struct AggregateResult {
    pub mean_net_pnl: Decimal,
    pub median_net_pnl: Decimal,
    pub var_95_net_pnl: Decimal, // Value at Risk (5th percentile)
    pub mean_fees: Decimal,
    pub mean_il: Decimal,
    pub iterations: usize,
}

impl<V: VolumeModel + Clone> MonteCarloRunner<V> {
    pub fn run(&mut self) -> AggregateResult {
        let mut results: Vec<SimulationResult> = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let gbm = GeometricBrownianMotion::new(
                self.initial_price,
                self.drift,
                self.volatility,
                self.time_step,
            );
            
            // Create a fresh volume model for each run if it has state
            let vol = self.volume_model.clone(); 
            
            let mut engine = SimulationEngine::new(
                self.position.clone(),
                gbm,
                vol,
                self.steps
            );
            
            results.push(engine.run());
        }
        
        self.aggregate(results)
    }
    
    fn aggregate(&self, results: Vec<SimulationResult>) -> AggregateResult {
        let count = Decimal::from(results.len());
        
        let total_pnl: Decimal = results.iter().map(|r| r.net_pnl).sum();
        let total_fees: Decimal = results.iter().map(|r| r.total_fees_earned).sum();
        let total_il: Decimal = results.iter().map(|r| r.total_il).sum();
        
        let mean_pnl = total_pnl / count;
        let mean_fees = total_fees / count;
        let mean_il = total_il / count;
        
        // Sort for percentiles
        let mut pnls: Vec<Decimal> = results.iter().map(|r| r.net_pnl).collect();
        pnls.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median_idx = results.len() / 2;
        let median_pnl = pnls[median_idx];
        
        // VaR 95% is the value at the 5th percentile
        let var_idx = (results.len() as f64 * 0.05).floor() as usize;
        let var_95 = pnls[var_idx.min(results.len() - 1)];
        
        AggregateResult {
            mean_net_pnl: mean_pnl,
            median_net_pnl: median_pnl,
            var_95_net_pnl: var_95,
            mean_fees,
            mean_il,
            iterations: results.len(),
        }
    }
}
