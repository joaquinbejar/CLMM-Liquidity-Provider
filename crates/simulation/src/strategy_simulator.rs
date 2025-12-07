//! Strategy-aware position simulator.
//!
//! This module provides simulation with integrated rebalancing strategies,
//! allowing for dynamic position management during backtests.

use crate::event::{EventLog, SimulationEvent};
use crate::liquidity::LiquidityModel;
use crate::price_path::PricePathGenerator;
use crate::state::{SimulationConfig, SimulationSummary};
use crate::strategies::{RebalanceAction, RebalanceReason, RebalanceStrategy, StrategyContext};
use crate::volume::VolumeModel;
use clmm_lp_domain::metrics::impermanent_loss::calculate_il_concentrated;
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive as _;

/// Result of a strategy simulation.
#[derive(Debug, Clone)]
pub struct StrategySimulationResult {
    /// Summary of the simulation.
    pub summary: SimulationSummary,
    /// Event log from the simulation.
    pub events: Vec<SimulationEvent>,
    /// Price path used.
    pub prices: Vec<Price>,
    /// Step-by-step PnL values.
    pub pnl_history: Vec<Decimal>,
    /// Step-by-step IL values.
    pub il_history: Vec<Decimal>,
    /// Step-by-step fee values.
    pub fee_history: Vec<Decimal>,
    /// Range history (step, range).
    pub range_history: Vec<(u64, PriceRange)>,
}

/// Simulates an LP position with a rebalancing strategy.
///
/// # Arguments
/// * `config` - Simulation configuration
/// * `price_path` - Price path generator
/// * `volume_model` - Volume model for fee calculation
/// * `liquidity_model` - Liquidity model
/// * `strategy` - Rebalancing strategy to use
///
/// # Returns
/// Simulation result with full metrics including rebalancing events
pub fn simulate_with_strategy<P, V, L, S>(
    config: &SimulationConfig,
    price_path: &mut P,
    volume_model: &mut V,
    liquidity_model: &L,
    strategy: &S,
) -> StrategySimulationResult
where
    P: PricePathGenerator,
    V: VolumeModel,
    L: LiquidityModel,
    S: RebalanceStrategy,
{
    let prices = price_path.generate(config.steps);

    if prices.is_empty() {
        return empty_result(config);
    }

    let entry_price = prices[0];
    let mut current_range = config.initial_range.clone();

    let mut event_log = EventLog::new();
    let mut cumulative_fees = Decimal::ZERO;
    let mut steps_in_range: u64 = 0;
    let mut max_il = Decimal::ZERO;
    let mut max_value = config.initial_capital;
    let mut max_drawdown = Decimal::ZERO;
    let mut rebalance_count: u32 = 0;
    let mut total_rebalance_cost = Decimal::ZERO;
    let mut steps_since_rebalance: u64 = 0;

    let mut pnl_history = Vec::with_capacity(prices.len());
    let mut il_history = Vec::with_capacity(prices.len());
    let mut fee_history = Vec::with_capacity(prices.len());
    let mut range_history = Vec::new();

    let mut was_in_range = is_in_range(&entry_price, &current_range);

    // Record initial range
    range_history.push((0, current_range.clone()));

    // Record position opened
    event_log.record(SimulationEvent::position_opened(
        0,
        entry_price,
        config.initial_capital,
        current_range.clone(),
    ));

    for (step, price) in prices.iter().enumerate() {
        let in_range = is_in_range(price, &current_range);

        // Track range transitions
        if in_range && !was_in_range {
            event_log.record(SimulationEvent::back_in_range(
                step as u64,
                *price,
                current_range.clone(),
            ));
        } else if !in_range && was_in_range {
            event_log.record(SimulationEvent::out_of_range(
                step as u64,
                *price,
                current_range.clone(),
            ));
        }
        was_in_range = in_range;

        // Calculate current IL for strategy context
        let il_decimal = calculate_il_concentrated(
            entry_price.value,
            price.value,
            current_range.lower_price.value,
            current_range.upper_price.value,
        )
        .unwrap_or(Decimal::ZERO);

        if il_decimal < max_il {
            max_il = il_decimal;
        }

        // Build strategy context
        let context = StrategyContext {
            current_price: *price,
            current_range: current_range.clone(),
            entry_price,
            steps_since_open: step as u64,
            steps_since_rebalance,
            current_il_pct: il_decimal,
            total_fees_earned: cumulative_fees,
        };

        // Evaluate strategy
        let action = strategy.evaluate(&context);

        match &action {
            RebalanceAction::Rebalance { new_range, reason } => {
                let old_range = current_range.clone();
                current_range = new_range.clone();
                rebalance_count += 1;
                total_rebalance_cost += config.rebalance_cost;
                steps_since_rebalance = 0;

                range_history.push((step as u64, current_range.clone()));

                event_log.record(SimulationEvent::rebalance(
                    step as u64,
                    *price,
                    old_range,
                    new_range.clone(),
                    format_reason(reason),
                    config.rebalance_cost,
                ));

                // Update in_range status after rebalance
                was_in_range = is_in_range(price, &current_range);
            }
            RebalanceAction::Close { reason: _ } => {
                // For close action, we stop earning fees but continue tracking
                event_log.record(SimulationEvent::position_closed(
                    step as u64,
                    *price,
                    config.initial_capital - (config.initial_capital * il_decimal.abs())
                        + cumulative_fees
                        - total_rebalance_cost,
                    cumulative_fees,
                    il_decimal,
                    cumulative_fees
                        - (config.initial_capital * il_decimal.abs())
                        - total_rebalance_cost,
                ));
                // Position is closed, skip remaining steps
                break;
            }
            RebalanceAction::Hold => {
                steps_since_rebalance += 1;
            }
        }

        // Calculate fees if in range
        let in_range_now = is_in_range(price, &current_range);
        if in_range_now {
            steps_in_range += 1;

            let volume = volume_model.get_volume(step);
            let pool_liquidity = liquidity_model.get_liquidity(step);

            let step_fees = if pool_liquidity > 0 {
                let lp_share = Decimal::from(config.pool_liquidity) / Decimal::from(pool_liquidity);
                volume * config.fee_rate * lp_share
            } else {
                Decimal::ZERO
            };

            cumulative_fees += step_fees;

            if step_fees > Decimal::ZERO {
                event_log.record(SimulationEvent::fee_collection(
                    step as u64,
                    *price,
                    step_fees,
                    cumulative_fees,
                ));
            }
        }

        // Calculate position value
        let il_amount = config.initial_capital * il_decimal.abs();
        let position_value =
            config.initial_capital - il_amount + cumulative_fees - total_rebalance_cost;
        let net_pnl = position_value - config.initial_capital;

        // Track max value and drawdown
        if position_value > max_value {
            max_value = position_value;
        }
        let drawdown = if max_value.is_zero() {
            Decimal::ZERO
        } else {
            (position_value - max_value) / max_value
        };
        if drawdown < max_drawdown {
            max_drawdown = drawdown;
        }

        pnl_history.push(net_pnl);
        il_history.push(il_decimal);
        fee_history.push(cumulative_fees);
    }

    let final_price = *prices.last().unwrap_or(&entry_price);

    let final_il_decimal = calculate_il_concentrated(
        entry_price.value,
        final_price.value,
        current_range.lower_price.value,
        current_range.upper_price.value,
    )
    .unwrap_or(Decimal::ZERO);

    let final_price_ratio = if entry_price.value.is_zero() {
        1.0
    } else {
        (final_price.value / entry_price.value)
            .to_f64()
            .unwrap_or(1.0)
    };

    let il_amount = config.initial_capital * final_il_decimal.abs();
    let final_value = config.initial_capital - il_amount + cumulative_fees - total_rebalance_cost;
    let net_pnl = final_value - config.initial_capital;
    let net_pnl_pct = if config.initial_capital.is_zero() {
        Decimal::ZERO
    } else {
        net_pnl / config.initial_capital
    };

    // HODL comparison
    let hodl_value =
        config.initial_capital * Decimal::try_from(final_price_ratio).unwrap_or(Decimal::ONE);
    let vs_hodl = final_value - hodl_value;

    // Record position closed if not already closed
    if !event_log
        .events()
        .iter()
        .any(|e| e.event_type == crate::event::SimulationEventType::PositionClosed)
    {
        event_log.record(SimulationEvent::position_closed(
            prices.len() as u64,
            final_price,
            final_value,
            cumulative_fees,
            final_il_decimal,
            net_pnl,
        ));
    }

    let summary = SimulationSummary {
        config: config.clone(),
        entry_price,
        final_price,
        total_steps: prices.len() as u64,
        steps_in_range,
        final_value,
        total_fees: cumulative_fees,
        final_il_pct: final_il_decimal,
        net_pnl,
        net_pnl_pct,
        rebalance_count,
        total_rebalance_cost,
        max_il_pct: max_il,
        max_drawdown_pct: max_drawdown,
        hodl_value,
        vs_hodl,
    };

    StrategySimulationResult {
        summary,
        events: event_log.events().to_vec(),
        prices,
        pnl_history,
        il_history,
        fee_history,
        range_history,
    }
}

/// Formats a rebalance reason as a string.
fn format_reason(reason: &RebalanceReason) -> String {
    match reason {
        RebalanceReason::Periodic { steps_elapsed } => {
            format!("Periodic rebalance after {} steps", steps_elapsed)
        }
        RebalanceReason::PriceThreshold { price_change_pct } => {
            format!("Price moved {}%", price_change_pct)
        }
        RebalanceReason::OutOfRange { current_price } => {
            format!("Price {} out of range", current_price)
        }
        RebalanceReason::ILThreshold { il_pct } => {
            format!("IL exceeded threshold: {}%", il_pct * Decimal::from(100))
        }
        RebalanceReason::Manual => "Manual rebalance".to_string(),
    }
}

/// Checks if a price is within a range.
fn is_in_range(price: &Price, range: &PriceRange) -> bool {
    price.value >= range.lower_price.value && price.value <= range.upper_price.value
}

/// Creates an empty result for edge cases.
fn empty_result(config: &SimulationConfig) -> StrategySimulationResult {
    let entry_price = Price::new(Decimal::ZERO);
    let summary = SimulationSummary {
        config: config.clone(),
        entry_price,
        final_price: entry_price,
        total_steps: 0,
        steps_in_range: 0,
        final_value: config.initial_capital,
        total_fees: Decimal::ZERO,
        final_il_pct: Decimal::ZERO,
        net_pnl: Decimal::ZERO,
        net_pnl_pct: Decimal::ZERO,
        rebalance_count: 0,
        total_rebalance_cost: Decimal::ZERO,
        max_il_pct: Decimal::ZERO,
        max_drawdown_pct: Decimal::ZERO,
        hodl_value: config.initial_capital,
        vs_hodl: Decimal::ZERO,
    };

    StrategySimulationResult {
        summary,
        events: Vec::new(),
        prices: Vec::new(),
        pnl_history: Vec::new(),
        il_history: Vec::new(),
        fee_history: Vec::new(),
        range_history: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liquidity::ConstantLiquidity;
    use crate::price_path::DeterministicPricePath;
    use crate::strategies::{PeriodicRebalance, StaticRange, ThresholdRebalance};
    use crate::volume::ConstantVolume;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulate_with_static_strategy() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(10)
            .with_fee_rate(dec!(0.003))
            .with_pool_liquidity(1_000_000);

        let prices = vec![dec!(100); 10];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);
        let strategy = StaticRange;

        let result = simulate_with_strategy(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
            &strategy,
        );

        assert_eq!(result.summary.total_steps, 10);
        assert_eq!(result.summary.rebalance_count, 0);
        assert!(result.summary.total_fees > Decimal::ZERO);
    }

    #[test]
    fn test_simulate_with_periodic_strategy() {
        let range = PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(20)
            .with_fee_rate(dec!(0.003))
            .with_rebalance_cost(dec!(1));

        let prices = vec![dec!(100); 20];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);
        let strategy = PeriodicRebalance::new(5, dec!(0.10)); // Rebalance every 5 steps

        let result = simulate_with_strategy(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
            &strategy,
        );

        // Should have rebalanced at steps 5, 10, 15
        assert!(result.summary.rebalance_count >= 3);
        assert!(result.summary.total_rebalance_cost >= dec!(3));
    }

    #[test]
    fn test_simulate_with_threshold_strategy() {
        let range = PriceRange::new(Price::new(dec!(95)), Price::new(dec!(105)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(5)
            .with_fee_rate(dec!(0.003))
            .with_rebalance_cost(dec!(1));

        // Price moves significantly
        let prices = vec![dec!(100), dec!(100), dec!(110), dec!(110), dec!(110)];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);
        let strategy = ThresholdRebalance::new(dec!(0.05), dec!(0.10)); // 5% threshold

        let result = simulate_with_strategy(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
            &strategy,
        );

        // Should have rebalanced when price moved out of range
        assert!(result.summary.rebalance_count >= 1);
        assert!(!result.range_history.is_empty());
    }

    #[test]
    fn test_range_history_tracking() {
        let range = PriceRange::new(Price::new(dec!(95)), Price::new(dec!(105)));
        let config = SimulationConfig::new(dec!(1000), range)
            .with_steps(10)
            .with_rebalance_cost(dec!(1));

        let prices = vec![dec!(100); 10];
        let mut price_path = DeterministicPricePath::new(prices);
        let mut volume_model = ConstantVolume::new(dec!(10000));
        let liquidity_model = ConstantLiquidity::new(1_000_000);
        let strategy = PeriodicRebalance::new(3, dec!(0.10));

        let result = simulate_with_strategy(
            &config,
            &mut price_path,
            &mut volume_model,
            &liquidity_model,
            &strategy,
        );

        // Should have initial range + rebalances
        assert!(!result.range_history.is_empty());
        // First entry should be at step 0
        assert_eq!(result.range_history[0].0, 0);
    }
}
