use clmm_lp_domain::value_objects::price::Price;
use rand_distr::{Distribution, Normal};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Trait for generating price paths.
pub trait PricePathGenerator {
    /// Generates a price path with the specified number of steps.
    fn generate(&mut self, steps: usize) -> Vec<Price>;
}

/// Geometric Brownian Motion price path generator.
pub struct GeometricBrownianMotion {
    /// The initial price.
    pub initial_price: Decimal,
    /// Annualized drift (mu).
    pub drift: f64, // annualized drift (mu)
    /// Annualized volatility (sigma).
    pub volatility: f64, // annualized volatility (sigma)
    /// Time step in years (dt).
    pub time_step: f64, // time step in years (dt) e.g. 1/365 for daily
}

impl GeometricBrownianMotion {
    /// Creates a new GeometricBrownianMotion generator.
    pub fn new(initial_price: Decimal, drift: f64, volatility: f64, time_step: f64) -> Self {
        Self {
            initial_price,
            drift,
            volatility,
            time_step,
        }
    }
}

impl PricePathGenerator for GeometricBrownianMotion {
    fn generate(&mut self, steps: usize) -> Vec<Price> {
        let mut prices = Vec::with_capacity(steps + 1);
        prices.push(Price::new(self.initial_price));

        let mut rng = rand::rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        let dt = self.time_step;
        let drift_term = (self.drift - 0.5 * self.volatility.powi(2)) * dt;
        let vol_term = self.volatility * dt.sqrt();

        let mut current_price = self.initial_price.to_f64().unwrap_or(0.0);

        for _ in 0..steps {
            let z = normal.sample(&mut rng);
            let change = (drift_term + vol_term * z).exp();
            current_price *= change;

            // Convert back to Decimal
            // Note: Standard f64 precision might drift from Decimal over simulated time,
            // but for Monte Carlo high performance, f64 is standard.
            // We cast back to Decimal for the domain object.
            let p = Decimal::from_f64(current_price).unwrap_or(Decimal::ZERO);
            prices.push(Price::new(p));
        }

        prices
    }
}

/// Deterministic price path generator (e.g., from historical data).
pub struct DeterministicPricePath {
    /// The sequence of prices.
    pub prices: Vec<Price>,
}

impl PricePathGenerator for DeterministicPricePath {
    fn generate(&mut self, _steps: usize) -> Vec<Price> {
        self.prices.clone()
    }
}

/// Historical price path generator that uses a provided list of historical prices.
/// This is similar to DeterministicPricePath but explicitly designed for backtesting flows
/// where we might want to iterate or slice the data.
pub struct HistoricalPricePath {
    /// The historical prices.
    pub prices: Vec<Price>,
}

impl HistoricalPricePath {
    /// Creates a new HistoricalPricePath.
    pub fn new(prices: Vec<Price>) -> Self {
        Self { prices }
    }
}

impl PricePathGenerator for HistoricalPricePath {
    fn generate(&mut self, _steps: usize) -> Vec<Price> {
        // For historical backtest, we usually want the exact sequence provided.
        // Steps argument might be used to limit or check length, but here we return the full available path
        // or up to steps if we wanted to enforce a limit.
        // For now, return full path.
        self.prices.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gbm_generation() {
        let initial = Decimal::from(100);
        let drift = 0.0;
        let vol = 0.2; // 20%
        let dt = 1.0 / 365.0; // daily

        let mut gbm = GeometricBrownianMotion::new(initial, drift, vol, dt);
        let path = gbm.generate(10);

        assert_eq!(path.len(), 11); // initial + 10 steps
        assert_eq!(path[0].value, initial);

        // Check that prices are not all same (unless vol is 0)
        let all_same = path.iter().all(|p| p.value == initial);
        assert!(!all_same);
    }
}
