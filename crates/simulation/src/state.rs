//! Simulation state management.
//!
//! This module provides structures for capturing and managing the state
//! of a simulation at any point in time.

use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;

/// Current state of a simulated pool.
#[derive(Debug, Clone)]
pub struct PoolState {
    /// Current price.
    pub current_price: Price,
    /// Current tick (if applicable).
    pub current_tick: Option<i32>,
    /// Total liquidity in the pool.
    pub total_liquidity: u128,
    /// 24-hour volume estimate.
    pub volume_24h: Decimal,
    /// Fee rate as decimal.
    pub fee_rate: Decimal,
}

impl PoolState {
    /// Creates a new pool state.
    #[must_use]
    pub fn new(current_price: Price, total_liquidity: u128, fee_rate: Decimal) -> Self {
        Self {
            current_price,
            current_tick: None,
            total_liquidity,
            volume_24h: Decimal::ZERO,
            fee_rate,
        }
    }

    /// Sets the current tick.
    #[must_use]
    pub fn with_tick(mut self, tick: i32) -> Self {
        self.current_tick = Some(tick);
        self
    }

    /// Sets the 24-hour volume.
    #[must_use]
    pub fn with_volume(mut self, volume: Decimal) -> Self {
        self.volume_24h = volume;
        self
    }
}

/// Current state of a simulated position.
#[derive(Debug, Clone)]
pub struct PositionState {
    /// Current price range.
    pub range: PriceRange,
    /// Position liquidity.
    pub liquidity: u128,
    /// Whether position is currently in range.
    pub in_range: bool,
    /// Entry price.
    pub entry_price: Price,
    /// Current position value in USD.
    pub value_usd: Decimal,
    /// Cumulative fees earned.
    pub fees_earned: Decimal,
    /// Current impermanent loss percentage.
    pub il_pct: Decimal,
    /// Net PnL.
    pub net_pnl: Decimal,
}

impl PositionState {
    /// Creates a new position state.
    #[must_use]
    pub fn new(range: PriceRange, entry_price: Price, initial_value: Decimal) -> Self {
        Self {
            range,
            liquidity: 0,
            in_range: true,
            entry_price,
            value_usd: initial_value,
            fees_earned: Decimal::ZERO,
            il_pct: Decimal::ZERO,
            net_pnl: Decimal::ZERO,
        }
    }

    /// Sets the liquidity.
    #[must_use]
    pub fn with_liquidity(mut self, liquidity: u128) -> Self {
        self.liquidity = liquidity;
        self
    }

    /// Checks if price is within the position range.
    #[must_use]
    pub fn is_price_in_range(&self, price: &Price) -> bool {
        price.value >= self.range.lower_price.value && price.value <= self.range.upper_price.value
    }
}

/// Complete simulation state at a point in time.
#[derive(Debug, Clone)]
pub struct SimulationState {
    /// Current step number.
    pub step: u64,
    /// Current timestamp (if available).
    pub timestamp: Option<u64>,
    /// Pool state.
    pub pool: PoolState,
    /// Position state.
    pub position: PositionState,
}

impl SimulationState {
    /// Creates a new simulation state.
    #[must_use]
    pub fn new(step: u64, pool: PoolState, position: PositionState) -> Self {
        Self {
            step,
            timestamp: None,
            pool,
            position,
        }
    }

    /// Sets the timestamp.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// Configuration for a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Initial capital in USD.
    pub initial_capital: Decimal,
    /// Initial price range.
    pub initial_range: PriceRange,
    /// Fee rate as decimal.
    pub fee_rate: Decimal,
    /// Pool liquidity.
    pub pool_liquidity: u128,
    /// Cost per rebalance transaction.
    pub rebalance_cost: Decimal,
    /// Number of simulation steps.
    pub steps: usize,
    /// Step duration in seconds (for time-based calculations).
    pub step_duration_seconds: u64,
}

impl SimulationConfig {
    /// Creates a new simulation config with defaults.
    #[must_use]
    pub fn new(initial_capital: Decimal, initial_range: PriceRange) -> Self {
        Self {
            initial_capital,
            initial_range,
            fee_rate: Decimal::new(3, 3), // 0.3%
            pool_liquidity: 1_000_000,
            rebalance_cost: Decimal::ONE,
            steps: 100,
            step_duration_seconds: 3600, // 1 hour
        }
    }

    /// Sets the fee rate.
    #[must_use]
    pub fn with_fee_rate(mut self, fee_rate: Decimal) -> Self {
        self.fee_rate = fee_rate;
        self
    }

    /// Sets the pool liquidity.
    #[must_use]
    pub fn with_pool_liquidity(mut self, liquidity: u128) -> Self {
        self.pool_liquidity = liquidity;
        self
    }

    /// Sets the rebalance cost.
    #[must_use]
    pub fn with_rebalance_cost(mut self, cost: Decimal) -> Self {
        self.rebalance_cost = cost;
        self
    }

    /// Sets the number of steps.
    #[must_use]
    pub fn with_steps(mut self, steps: usize) -> Self {
        self.steps = steps;
        self
    }

    /// Sets the step duration.
    #[must_use]
    pub fn with_step_duration(mut self, seconds: u64) -> Self {
        self.step_duration_seconds = seconds;
        self
    }

    /// Returns total simulation duration in seconds.
    #[must_use]
    pub fn total_duration_seconds(&self) -> u64 {
        self.steps as u64 * self.step_duration_seconds
    }

    /// Returns total simulation duration in days.
    #[must_use]
    pub fn total_duration_days(&self) -> f64 {
        self.total_duration_seconds() as f64 / 86400.0
    }
}

/// Results from a completed simulation.
#[derive(Debug, Clone)]
pub struct SimulationSummary {
    /// Configuration used.
    pub config: SimulationConfig,
    /// Entry price.
    pub entry_price: Price,
    /// Final price.
    pub final_price: Price,
    /// Total steps executed.
    pub total_steps: u64,
    /// Steps where position was in range.
    pub steps_in_range: u64,
    /// Final position value.
    pub final_value: Decimal,
    /// Total fees earned.
    pub total_fees: Decimal,
    /// Final IL percentage.
    pub final_il_pct: Decimal,
    /// Net PnL.
    pub net_pnl: Decimal,
    /// Net PnL percentage.
    pub net_pnl_pct: Decimal,
    /// Number of rebalances.
    pub rebalance_count: u32,
    /// Total rebalance costs.
    pub total_rebalance_cost: Decimal,
    /// Maximum IL observed.
    pub max_il_pct: Decimal,
    /// Maximum drawdown.
    pub max_drawdown_pct: Decimal,
    /// HODL value for comparison.
    pub hodl_value: Decimal,
    /// Performance vs HODL.
    pub vs_hodl: Decimal,
}

impl SimulationSummary {
    /// Returns the percentage of time in range.
    #[must_use]
    pub fn time_in_range_pct(&self) -> Decimal {
        if self.total_steps == 0 {
            return Decimal::ZERO;
        }
        Decimal::from(self.steps_in_range) / Decimal::from(self.total_steps)
    }

    /// Returns the annualized return.
    #[must_use]
    pub fn annualized_return(&self) -> Decimal {
        let days = self.config.total_duration_days();
        if days <= 0.0 || self.config.initial_capital.is_zero() {
            return Decimal::ZERO;
        }

        let roi = self.net_pnl / self.config.initial_capital;
        roi * Decimal::from(365) / Decimal::try_from(days).unwrap_or(Decimal::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pool_state_creation() {
        let state = PoolState::new(Price::new(dec!(100)), 1_000_000, dec!(0.003))
            .with_tick(-100)
            .with_volume(dec!(500000));

        assert_eq!(state.current_price.value, dec!(100));
        assert_eq!(state.current_tick, Some(-100));
        assert_eq!(state.volume_24h, dec!(500000));
    }

    #[test]
    fn test_position_state_in_range() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let state = PositionState::new(range, Price::new(dec!(100)), dec!(1000));

        assert!(state.is_price_in_range(&Price::new(dec!(100))));
        assert!(state.is_price_in_range(&Price::new(dec!(90))));
        assert!(state.is_price_in_range(&Price::new(dec!(110))));
        assert!(!state.is_price_in_range(&Price::new(dec!(89))));
        assert!(!state.is_price_in_range(&Price::new(dec!(111))));
    }

    #[test]
    fn test_simulation_config_duration() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(720) // 30 days of hourly data
            .with_step_duration(3600);

        assert_eq!(config.total_duration_seconds(), 720 * 3600);
        assert!((config.total_duration_days() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_simulation_summary_time_in_range() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let config = SimulationConfig::new(dec!(1000), range);

        let summary = SimulationSummary {
            config,
            entry_price: Price::new(dec!(100)),
            final_price: Price::new(dec!(105)),
            total_steps: 100,
            steps_in_range: 80,
            final_value: dec!(1050),
            total_fees: dec!(100),
            final_il_pct: dec!(-0.02),
            net_pnl: dec!(50),
            net_pnl_pct: dec!(0.05),
            rebalance_count: 2,
            total_rebalance_cost: dec!(2),
            max_il_pct: dec!(-0.05),
            max_drawdown_pct: dec!(-0.03),
            hodl_value: dec!(1025),
            vs_hodl: dec!(25),
        };

        assert_eq!(summary.time_in_range_pct(), dec!(0.8));
    }
}
